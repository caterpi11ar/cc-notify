use super::super::{lock_conn, to_json_string, Database};
use crate::error::AppError;
use crate::models::{
    ApiKeyStatus, GatewayLog, NewGatewayLog, Provider, ProviderInput, ProviderProtocol,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use rusqlite::{params, OptionalExtension, Row};
use sha2::{Digest, Sha256};
use std::str::FromStr;
use uuid::Uuid;

const LOCAL_API_KEY_HASH: &str = "gateway.local_api_key_hash";
const LOCAL_API_KEY_PREVIEW: &str = "gateway.local_api_key_preview";
const GATEWAY_PORT: &str = "gateway.port";

impl Database {
    pub fn get_gateway_port(&self) -> Result<u16, AppError> {
        let port = self
            .get_setting(GATEWAY_PORT)?
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(17777);
        Ok(port)
    }

    pub fn set_gateway_port(&self, port: u16) -> Result<(), AppError> {
        self.set_setting(GATEWAY_PORT, &port.to_string())
    }

    pub fn ensure_local_api_key(&self) -> Result<String, AppError> {
        if self.get_setting(LOCAL_API_KEY_HASH)?.is_some() {
            return Ok(self.get_setting(LOCAL_API_KEY_PREVIEW)?.unwrap_or_default());
        }
        self.regenerate_local_api_key()
    }

    pub fn regenerate_local_api_key(&self) -> Result<String, AppError> {
        let key = generate_api_key();
        let preview = preview_key(&key);
        self.set_setting(LOCAL_API_KEY_HASH, &hash_secret(&key))?;
        self.set_setting(LOCAL_API_KEY_PREVIEW, &preview)?;
        self.set_setting("gateway.local_api_key_plain_once", &key)?;
        Ok(key)
    }

    pub fn get_local_api_key_status(&self) -> Result<ApiKeyStatus, AppError> {
        let exists = self.get_setting(LOCAL_API_KEY_HASH)?.is_some();
        let preview = self.get_setting(LOCAL_API_KEY_PREVIEW)?;
        Ok(ApiKeyStatus { exists, preview })
    }

    pub fn get_local_api_key_once(&self) -> Result<Option<String>, AppError> {
        self.get_setting("gateway.local_api_key_plain_once")
    }

    pub fn validate_local_api_key(&self, key: &str) -> Result<bool, AppError> {
        let Some(stored_hash) = self.get_setting(LOCAL_API_KEY_HASH)? else {
            return Ok(false);
        };
        Ok(hash_secret(key) == stored_hash)
    }

    pub fn get_providers(&self) -> Result<Vec<Provider>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn.prepare(
            "SELECT id, name, base_url, protocol, enabled, default_model, models,
                    model_prefixes, timeout_ms, max_tokens_field, secret_ref,
                    has_api_key, health_status, created_at, updated_at
             FROM providers ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], provider_from_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn get_provider(&self, id: &str) -> Result<Option<Provider>, AppError> {
        let conn = lock_conn!(self.conn);
        conn.query_row(
            "SELECT id, name, base_url, protocol, enabled, default_model, models,
                    model_prefixes, timeout_ms, max_tokens_field, secret_ref,
                    has_api_key, health_status, created_at, updated_at
             FROM providers WHERE id = ?1",
            [id],
            provider_from_row,
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn create_provider(&self, input: ProviderInput) -> Result<Provider, AppError> {
        let id = Uuid::new_v4().to_string();
        let secret_ref = input
            .api_key
            .as_ref()
            .filter(|key| !key.is_empty())
            .map(|_| format!("provider:{id}:api_key"));
        let conn = lock_conn!(self.conn);
        conn.execute(
            "INSERT INTO providers
             (id, name, base_url, protocol, enabled, default_model, models, model_prefixes,
              timeout_ms, max_tokens_field, secret_ref, has_api_key, health_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 'unknown')",
            params![
                id,
                input.name,
                trim_slash(&input.base_url),
                input.protocol.as_str(),
                input.enabled,
                input.default_model,
                to_json_string(&input.models)?,
                to_json_string(&input.model_prefixes)?,
                input.timeout_ms,
                input.max_tokens_field,
                secret_ref,
                input.api_key.as_ref().is_some_and(|key| !key.is_empty()),
            ],
        )?;
        if let (Some(secret_ref), Some(api_key)) = (&secret_ref, input.api_key) {
            conn.execute(
                "INSERT INTO provider_secrets (secret_ref, provider_id, secret)
                 VALUES (?1, ?2, ?3)",
                params![secret_ref, id, api_key],
            )?;
        }
        drop(conn);
        self.get_provider(&id)?
            .ok_or_else(|| AppError::Database("Created provider could not be loaded".to_string()))
    }

    pub fn get_provider_secret(&self, secret_ref: &str) -> Result<Option<String>, AppError> {
        let conn = lock_conn!(self.conn);
        conn.query_row(
            "SELECT secret FROM provider_secrets WHERE secret_ref = ?1",
            [secret_ref],
            |row| row.get(0),
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn insert_gateway_log(&self, log: NewGatewayLog) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "INSERT INTO gateway_logs
             (client_type, endpoint, requested_model, resolved_provider, resolved_model,
              protocol_in, protocol_out, status_code, latency_ms, stream, token_estimate,
              input_tokens, output_tokens, total_tokens, cache_creation_input_tokens,
              cache_read_input_tokens, fidelity_mode, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                log.client_type,
                log.endpoint,
                log.requested_model,
                log.resolved_provider,
                log.resolved_model,
                log.protocol_in,
                log.protocol_out,
                log.status_code,
                log.latency_ms,
                log.stream,
                log.token_estimate,
                log.input_tokens,
                log.output_tokens,
                log.total_tokens,
                log.cache_creation_input_tokens,
                log.cache_read_input_tokens,
                log.fidelity_mode,
                log.error_message,
            ],
        )?;
        Ok(())
    }

    pub fn get_gateway_logs(&self, limit: i64) -> Result<Vec<GatewayLog>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn.prepare(
            "SELECT id, time, client_type, endpoint, requested_model, resolved_provider,
                    resolved_model, protocol_in, protocol_out, status_code, latency_ms,
                    stream, token_estimate, input_tokens, output_tokens, total_tokens,
                    cache_creation_input_tokens, cache_read_input_tokens, fidelity_mode,
                    error_message
             FROM gateway_logs ORDER BY time DESC, id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit], |row| {
            Ok(GatewayLog {
                id: row.get(0)?,
                time: row.get(1)?,
                client_type: row.get(2)?,
                endpoint: row.get(3)?,
                requested_model: row.get(4)?,
                resolved_provider: row.get(5)?,
                resolved_model: row.get(6)?,
                protocol_in: row.get(7)?,
                protocol_out: row.get(8)?,
                status_code: row.get(9)?,
                latency_ms: row.get(10)?,
                stream: row.get(11)?,
                token_estimate: row.get(12)?,
                input_tokens: row.get(13)?,
                output_tokens: row.get(14)?,
                total_tokens: row.get(15)?,
                cache_creation_input_tokens: row.get(16)?,
                cache_read_input_tokens: row.get(17)?,
                fidelity_mode: row.get(18)?,
                error_message: row.get(19)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn clear_gateway_logs(&self) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute("DELETE FROM gateway_logs", [])?;
        Ok(())
    }
}

fn provider_from_row(row: &Row<'_>) -> rusqlite::Result<Provider> {
    let protocol: String = row.get(3)?;
    let models: String = row.get(6)?;
    let model_prefixes: String = row.get(7)?;
    Ok(Provider {
        id: row.get(0)?,
        name: row.get(1)?,
        base_url: row.get(2)?,
        protocol: ProviderProtocol::from_str(&protocol)
            .unwrap_or(ProviderProtocol::OpenAiCompatible),
        enabled: row.get(4)?,
        default_model: row.get(5)?,
        models: serde_json::from_str(&models).unwrap_or_default(),
        model_prefixes: serde_json::from_str(&model_prefixes).unwrap_or_default(),
        timeout_ms: row.get(8)?,
        max_tokens_field: row.get(9)?,
        secret_ref: row.get(10)?,
        has_api_key: row.get(11)?,
        health_status: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

fn generate_api_key() -> String {
    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    format!("lgw_{}", URL_SAFE_NO_PAD.encode(bytes))
}

fn hash_secret(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn preview_key(key: &str) -> String {
    if key.len() <= 12 {
        return key.to_string();
    }
    format!("{}...{}", &key[..8], &key[key.len() - 4..])
}

fn trim_slash(value: &str) -> String {
    value.trim().trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProviderProtocol;

    #[test]
    fn validates_local_api_key() {
        let db = Database::memory().unwrap();
        let key = db.regenerate_local_api_key().unwrap();
        assert!(db.validate_local_api_key(&key).unwrap());
        assert!(!db.validate_local_api_key("wrong").unwrap());
    }

    #[test]
    fn provider_crud_keeps_secret_out_of_provider() {
        let db = Database::memory().unwrap();
        let provider = db
            .create_provider(ProviderInput {
                name: "Local".to_string(),
                base_url: "http://127.0.0.1:11434/v1/".to_string(),
                protocol: ProviderProtocol::OpenAiCompatible,
                enabled: true,
                default_model: "llama3".to_string(),
                models: vec!["llama3".to_string()],
                model_prefixes: vec!["llama".to_string()],
                timeout_ms: 30000,
                max_tokens_field: "max_tokens".to_string(),
                api_key: Some("secret".to_string()),
            })
            .unwrap();
        assert!(provider.has_api_key);
        let secret_ref = provider.secret_ref.unwrap();
        assert_eq!(
            db.get_provider_secret(&secret_ref).unwrap(),
            Some("secret".to_string())
        );
    }
}

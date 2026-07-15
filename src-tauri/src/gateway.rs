use crate::database::Database;
use crate::error::AppError;
use crate::models::{GatewayStatus, NewGatewayLog, ProviderProtocol};
use crate::protocol;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Instant;

pub struct GatewayRuntime {
    handle: Mutex<Option<GatewayHandle>>,
}

struct GatewayHandle {
    port: u16,
    shutdown: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

#[derive(Debug, Clone, Default)]
struct UsageMetrics {
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
    total_tokens: Option<i64>,
    cache_creation_input_tokens: Option<i64>,
    cache_read_input_tokens: Option<i64>,
}

impl GatewayRuntime {
    pub fn new() -> Self {
        Self {
            handle: Mutex::new(None),
        }
    }

    pub fn status(&self, db: &Arc<Database>) -> Result<GatewayStatus, AppError> {
        let configured_port = db.get_gateway_port()?;
        let guard = self.handle.lock()?;
        let running_port = guard.as_ref().map(|handle| handle.port);
        let port = running_port.unwrap_or(configured_port);
        Ok(status_for(port, running_port.is_some()))
    }

    pub fn start(&self, db: Arc<Database>) -> Result<GatewayStatus, AppError> {
        let mut guard = self.handle.lock()?;
        if let Some(handle) = guard.as_ref() {
            return Ok(status_for(handle.port, true));
        }

        db.ensure_local_api_key()?;
        let port = db.get_gateway_port()?;
        let listener = TcpListener::bind(("127.0.0.1", port))
            .map_err(|e| AppError::Message(format!("Failed to bind 127.0.0.1:{port}: {e}")))?;
        listener
            .set_nonblocking(false)
            .map_err(|e| AppError::Message(format!("Failed to configure listener: {e}")))?;

        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_thread = shutdown.clone();
        let thread = thread::spawn(move || {
            for stream in listener.incoming() {
                if shutdown_thread.load(Ordering::Relaxed) {
                    break;
                }
                match stream {
                    Ok(stream) => {
                        let db = db.clone();
                        thread::spawn(move || {
                            if let Err(error) = handle_stream(stream, db) {
                                log::warn!("Gateway request failed: {error}");
                            }
                        });
                    }
                    Err(error) => {
                        log::warn!("Gateway accept failed: {error}");
                    }
                }
            }
        });

        *guard = Some(GatewayHandle {
            port,
            shutdown,
            thread,
        });
        Ok(status_for(port, true))
    }

    pub fn stop(&self) -> Result<(), AppError> {
        let mut guard = self.handle.lock()?;
        if let Some(handle) = guard.take() {
            handle.shutdown.store(true, Ordering::Relaxed);
            let _ = TcpStream::connect(("127.0.0.1", handle.port));
            let _ = handle.thread.join();
        }
        Ok(())
    }

    pub fn restart(&self, db: Arc<Database>) -> Result<GatewayStatus, AppError> {
        self.stop()?;
        self.start(db)
    }
}

fn handle_stream(mut stream: TcpStream, db: Arc<Database>) -> Result<(), AppError> {
    let mut reader = BufReader::new(
        stream
            .try_clone()
            .map_err(|e| AppError::Message(e.to_string()))?,
    );
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|e| AppError::Message(e.to_string()))?;
    if request_line.trim().is_empty() {
        return Ok(());
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        write_json(&mut stream, 400, &json!({ "error": "bad request" }))?;
        return Ok(());
    }
    let method = parts[0].to_string();
    let raw_path = parts[1].to_string();
    let path = raw_path
        .split_once('?')
        .map(|(path, _)| path.to_string())
        .unwrap_or_else(|| raw_path.clone());

    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| AppError::Message(e.to_string()))?;
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.to_ascii_lowercase(), value.trim().to_string());
        }
    }

    let content_length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = vec![0_u8; content_length];
    if content_length > 0 {
        reader
            .read_exact(&mut body)
            .map_err(|e| AppError::Message(e.to_string()))?;
    }

    if method == "OPTIONS" {
        write_empty(&mut stream, 204)?;
        return Ok(());
    }

    let Some(api_key) = extract_api_key(&headers) else {
        log_request(
            &db,
            "unknown",
            &path,
            "",
            "unknown",
            None,
            401,
            0,
            false,
            Some("missing API key".to_string()),
        );
        write_json(
            &mut stream,
            401,
            &json!({ "error": { "message": "Missing local API key" } }),
        )?;
        return Ok(());
    };

    if !db.validate_local_api_key(&api_key)? {
        log_request(
            &db,
            "unknown",
            &path,
            "",
            "unknown",
            None,
            401,
            0,
            false,
            Some("invalid API key".to_string()),
        );
        write_json(
            &mut stream,
            401,
            &json!({ "error": { "message": "Invalid local API key" } }),
        )?;
        return Ok(());
    }

    match (method.as_str(), path.as_str()) {
        ("GET", "/health") => write_json(&mut stream, 200, &json!({ "ok": true })),
        _ if path == "/v1" || path.starts_with("/v1/") => {
            handle_transparent_proxy(&mut stream, db, &method, &raw_path, &path, &headers, &body)
        }
        _ => write_json(
            &mut stream,
            404,
            &json!({ "error": { "message": "Not found" } }),
        ),
    }
}

fn handle_transparent_proxy(
    stream: &mut TcpStream,
    db: Arc<Database>,
    method: &str,
    raw_path: &str,
    path: &str,
    incoming_headers: &HashMap<String, String>,
    body: &[u8],
) -> Result<(), AppError> {
    let started = Instant::now();
    let request = serde_json::from_slice::<Value>(body).ok();
    let requested_model = request
        .as_ref()
        .map(protocol::request_model)
        .unwrap_or_default();
    let resolved_model = requested_model.clone();
    let stream_requested = request.as_ref().is_some_and(protocol::request_stream);
    let token_estimate = request.as_ref().map(protocol::estimate_tokens);
    let providers = db.get_providers()?;
    let Some(provider) = providers.into_iter().find(|provider| provider.enabled) else {
        log_request(
            &db,
            "proxy",
            path,
            &requested_model,
            "openai",
            None,
            400,
            started.elapsed().as_millis() as i64,
            stream_requested,
            Some("No enabled upstream is configured".to_string()),
        );
        write_json(
            stream,
            400,
            &json!({ "error": { "message": "No enabled upstream is configured" } }),
        )?;
        return Ok(());
    };

    let upstream_url = upstream_url_for(&provider.base_url, raw_path);
    let timeout = std::time::Duration::from_millis(provider.timeout_ms.max(1000) as u64);
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| AppError::Message(format!("HTTP client error: {e}")))?;
    let req_method = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|e| AppError::InvalidInput(format!("Invalid HTTP method: {e}")))?;
    let mut request_builder = client
        .request(req_method, upstream_url)
        .headers(proxy_headers(&db, &provider, incoming_headers)?);
    if !body.is_empty() {
        request_builder = request_builder.body(body.to_vec());
    }

    let response = request_builder
        .send()
        .map_err(|e| AppError::Message(format!("Upstream request failed: {e}")))?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let bytes = response
        .bytes()
        .map_err(|e| AppError::Message(format!("Upstream response read failed: {e}")))?;
    let body_value = serde_json::from_slice::<Value>(&bytes).ok();
    let usage = body_value
        .as_ref()
        .map(extract_usage_metrics)
        .unwrap_or_default();
    let error_message = if status >= 400 {
        body_value
            .as_ref()
            .and_then(|body| body.pointer("/error/message").and_then(Value::as_str))
            .or_else(|| {
                body_value
                    .as_ref()
                    .and_then(|body| body.get("message").and_then(Value::as_str))
            })
            .map(ToString::to_string)
    } else {
        None
    };

    log_request_full(
        &db,
        NewGatewayLog {
            client_type: protocol_for_path(path).to_string(),
            endpoint: path.to_string(),
            requested_model,
            resolved_provider: Some(provider.name),
            resolved_model: if resolved_model.is_empty() {
                None
            } else {
                Some(resolved_model)
            },
            protocol_in: protocol_for_path(path).to_string(),
            protocol_out: Some(protocol_for_provider(&provider.protocol).to_string()),
            status_code: status as i64,
            latency_ms: started.elapsed().as_millis() as i64,
            stream: stream_requested,
            token_estimate,
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            total_tokens: usage.total_tokens,
            cache_creation_input_tokens: usage.cache_creation_input_tokens,
            cache_read_input_tokens: usage.cache_read_input_tokens,
            fidelity_mode: "proxy".to_string(),
            error_message,
        },
    );
    write_proxy_response(stream, status, &content_type, &bytes)
}

fn upstream_url_for(base_url: &str, raw_path: &str) -> String {
    let suffix = raw_path.strip_prefix("/v1").unwrap_or(raw_path);
    if suffix.is_empty() {
        base_url.trim_end_matches('/').to_string()
    } else {
        format!("{}{}", base_url.trim_end_matches('/'), suffix)
    }
}

fn protocol_for_path(path: &str) -> &'static str {
    if path == "/v1/messages" {
        "anthropic"
    } else {
        "openai"
    }
}

fn protocol_for_provider(protocol: &ProviderProtocol) -> &'static str {
    match protocol {
        ProviderProtocol::AnthropicCompatible => "anthropic",
        _ => "openai",
    }
}

fn proxy_headers(
    db: &Arc<Database>,
    provider: &crate::models::Provider,
    incoming_headers: &HashMap<String, String>,
) -> Result<HeaderMap, AppError> {
    let mut headers = HeaderMap::new();
    if let Some(content_type) = incoming_headers.get("content-type") {
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(content_type).map_err(|e| AppError::Message(e.to_string()))?,
        );
    } else {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    }
    if let Some(accept) = incoming_headers.get("accept") {
        headers.insert(
            "accept",
            HeaderValue::from_str(accept).map_err(|e| AppError::Message(e.to_string()))?,
        );
    }
    if provider.protocol == ProviderProtocol::AnthropicCompatible {
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    }
    if let Some(secret_ref) = &provider.secret_ref {
        if let Some(api_key) = db.get_provider_secret(secret_ref)? {
            if !api_key.is_empty() {
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {api_key}"))
                        .map_err(|e| AppError::Message(e.to_string()))?,
                );
                headers.insert(
                    "x-api-key",
                    HeaderValue::from_str(&api_key)
                        .map_err(|e| AppError::Message(e.to_string()))?,
                );
            }
        }
    }
    Ok(headers)
}

fn extract_api_key(headers: &HashMap<String, String>) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|value| value.strip_prefix("Bearer ").map(ToString::to_string))
        .or_else(|| headers.get("x-api-key").cloned())
}

fn status_for(port: u16, running: bool) -> GatewayStatus {
    GatewayStatus {
        running,
        host: "127.0.0.1".to_string(),
        port,
        openai_base_url: format!("http://127.0.0.1:{port}/v1"),
        anthropic_base_url: format!("http://127.0.0.1:{port}"),
    }
}

fn write_json(stream: &mut TcpStream, status: u16, body: &Value) -> Result<(), AppError> {
    let bytes = serde_json::to_vec(body).map_err(|e| AppError::Message(e.to_string()))?;
    let reason = reason(status);
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: authorization,x-api-key,content-type,anthropic-version\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        bytes.len()
    )
    .map_err(|e| AppError::Message(e.to_string()))?;
    stream
        .write_all(&bytes)
        .map_err(|e| AppError::Message(e.to_string()))
}

fn write_empty(stream: &mut TcpStream, status: u16) -> Result<(), AppError> {
    let reason = reason(status);
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: authorization,x-api-key,content-type,anthropic-version\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
    .map_err(|e| AppError::Message(e.to_string()))
}

fn write_proxy_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), AppError> {
    let reason = reason(status);
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: authorization,x-api-key,content-type,anthropic-version\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .map_err(|e| AppError::Message(e.to_string()))?;
    stream
        .write_all(body)
        .map_err(|e| AppError::Message(e.to_string()))
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        502 => "Bad Gateway",
        _ => "OK",
    }
}

fn log_request(
    db: &Arc<Database>,
    client_type: &str,
    endpoint: &str,
    requested_model: &str,
    protocol_in: &str,
    protocol_out: Option<String>,
    status_code: i64,
    latency_ms: i64,
    stream: bool,
    error_message: Option<String>,
) {
    log_request_full(
        db,
        NewGatewayLog {
            client_type: client_type.to_string(),
            endpoint: endpoint.to_string(),
            requested_model: requested_model.to_string(),
            resolved_provider: None,
            resolved_model: None,
            protocol_in: protocol_in.to_string(),
            protocol_out,
            status_code,
            latency_ms,
            stream,
            token_estimate: None,
            input_tokens: None,
            output_tokens: None,
            total_tokens: None,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
            fidelity_mode: "strict".to_string(),
            error_message,
        },
    );
}

fn extract_usage_metrics(body: &Value) -> UsageMetrics {
    let Some(usage) = body.get("usage") else {
        return UsageMetrics::default();
    };

    let input_tokens = first_i64(usage, &["prompt_tokens", "input_tokens"]);
    let output_tokens = first_i64(usage, &["completion_tokens", "output_tokens"]);
    let cache_creation_input_tokens =
        first_i64(usage, &["cache_creation_input_tokens"]).or_else(|| {
            usage
                .pointer("/prompt_tokens_details/cached_tokens")
                .and_then(Value::as_i64)
        });
    let cache_read_input_tokens = first_i64(usage, &["cache_read_input_tokens"]);
    let total_tokens =
        first_i64(usage, &["total_tokens"]).or_else(|| match (input_tokens, output_tokens) {
            (Some(input), Some(output)) => Some(input + output),
            _ => None,
        });

    UsageMetrics {
        input_tokens,
        output_tokens,
        total_tokens,
        cache_creation_input_tokens,
        cache_read_input_tokens,
    }
}

fn first_i64(value: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_i64))
}

fn log_request_full(db: &Arc<Database>, entry: NewGatewayLog) {
    if let Err(error) = db.insert_gateway_log(entry) {
        log::warn!("Failed to write gateway log: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ProviderInput, ProviderProtocol};
    use std::net::TcpListener;
    use std::sync::mpsc;

    #[test]
    fn rejects_missing_and_invalid_local_api_key() {
        let db = Arc::new(Database::memory().unwrap());
        let gateway = GatewayRuntime::new();
        let port = free_port();
        db.set_gateway_port(port).unwrap();
        let key = db.regenerate_local_api_key().unwrap();
        let (upstream_url, _received) = mock_json_upstream(json!({
            "object": "list",
            "data": []
        }));
        db.create_provider(openai_provider("Mock OpenAI", &upstream_url))
            .unwrap();

        gateway.start(db.clone()).unwrap();
        let client = reqwest::blocking::Client::new();
        let missing = client
            .get(format!("http://127.0.0.1:{port}/v1/models"))
            .send()
            .unwrap();
        let invalid = client
            .get(format!("http://127.0.0.1:{port}/v1/models"))
            .bearer_auth("wrong")
            .send()
            .unwrap();
        let valid = client
            .get(format!("http://127.0.0.1:{port}/v1/models"))
            .bearer_auth(key)
            .send()
            .unwrap();
        gateway.stop().unwrap();

        assert_eq!(missing.status().as_u16(), 401);
        assert_eq!(invalid.status().as_u16(), 401);
        assert_eq!(valid.status().as_u16(), 200);
    }

    #[test]
    fn proxies_models_to_upstream() {
        let (upstream_url, received) = mock_json_upstream(json!({
            "object": "list",
            "data": [{ "id": "llama3", "object": "model" }]
        }));
        let db = Arc::new(Database::memory().unwrap());
        let gateway = GatewayRuntime::new();
        let port = free_port();
        db.set_gateway_port(port).unwrap();
        let key = db.regenerate_local_api_key().unwrap();
        db.create_provider(openai_provider("Mock OpenAI", &upstream_url))
            .unwrap();

        gateway.start(db.clone()).unwrap();
        let response: Value = reqwest::blocking::Client::new()
            .get(format!("http://127.0.0.1:{port}/v1/models"))
            .header("x-api-key", key)
            .send()
            .unwrap()
            .json()
            .unwrap();
        gateway.stop().unwrap();

        let upstream_request = received.recv().unwrap();
        assert!(upstream_request.starts_with("GET /v1/models "));
        assert_eq!(response["data"][0]["id"], "llama3");
        let logs = db.get_gateway_logs(10).unwrap();
        assert_eq!(logs[0].endpoint, "/v1/models");
        assert_eq!(logs[0].resolved_provider.as_deref(), Some("Mock OpenAI"));
    }

    #[test]
    fn forwards_openai_chat_completion_and_writes_log() {
        let (upstream_url, received) = mock_json_upstream(json!({
            "id": "chatcmpl-test",
            "choices": [{ "message": { "role": "assistant", "content": "pong" } }],
            "usage": { "prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2 }
        }));
        let db = Arc::new(Database::memory().unwrap());
        let gateway = GatewayRuntime::new();
        let port = free_port();
        db.set_gateway_port(port).unwrap();
        let key = db.regenerate_local_api_key().unwrap();
        db.create_provider(openai_provider("Mock OpenAI", &upstream_url))
            .unwrap();

        gateway.start(db.clone()).unwrap();
        let response: Value = reqwest::blocking::Client::new()
            .post(format!("http://127.0.0.1:{port}/v1/chat/completions"))
            .bearer_auth(key)
            .json(&json!({
                "model": "llama3",
                "messages": [{ "role": "user", "content": "ping" }]
            }))
            .send()
            .unwrap()
            .json()
            .unwrap();
        gateway.stop().unwrap();

        let upstream_request = received.recv().unwrap();
        assert!(upstream_request.starts_with("POST /v1/chat/completions "));
        assert_eq!(response["choices"][0]["message"]["content"], "pong");
        let logs = db.get_gateway_logs(10).unwrap();
        assert_eq!(logs[0].resolved_provider.as_deref(), Some("Mock OpenAI"));
        assert_eq!(logs[0].status_code, 200);
        assert_eq!(logs[0].input_tokens, Some(1));
        assert_eq!(logs[0].output_tokens, Some(1));
        assert_eq!(logs[0].total_tokens, Some(2));
    }

    #[test]
    fn proxies_anthropic_messages_without_conversion() {
        let (upstream_url, received) = mock_json_upstream(json!({
            "id": "msg-test",
            "type": "message",
            "content": [{ "type": "text", "text": "hello" }],
            "usage": { "input_tokens": 1, "output_tokens": 1 }
        }));
        let db = Arc::new(Database::memory().unwrap());
        let gateway = GatewayRuntime::new();
        let port = free_port();
        db.set_gateway_port(port).unwrap();
        let key = db.regenerate_local_api_key().unwrap();
        db.create_provider(openai_provider("Mock OpenAI", &upstream_url))
            .unwrap();

        gateway.start(db.clone()).unwrap();
        let response: Value = reqwest::blocking::Client::new()
            .post(format!("http://127.0.0.1:{port}/v1/messages"))
            .header("x-api-key", key)
            .json(&json!({
                "model": "llama3",
                "system": "Be terse.",
                "messages": [{ "role": "user", "content": "hi" }],
                "max_tokens": 32
            }))
            .send()
            .unwrap()
            .json()
            .unwrap();
        gateway.stop().unwrap();

        let upstream_request = received.recv().unwrap();
        assert!(upstream_request.starts_with("POST /v1/messages "));
        assert!(upstream_request.contains(r#""system":"Be terse.""#));
        assert_eq!(response["type"], "message");
        assert_eq!(response["content"][0]["text"], "hello");
        let logs = db.get_gateway_logs(10).unwrap();
        assert_eq!(logs[0].protocol_in, "anthropic");
        assert_eq!(logs[0].protocol_out.as_deref(), Some("openai"));
        assert_eq!(logs[0].input_tokens, Some(1));
        assert_eq!(logs[0].output_tokens, Some(1));
        assert_eq!(logs[0].total_tokens, Some(2));
    }

    #[test]
    fn forwards_anthropic_messages_to_anthropic_upstream_and_writes_usage() {
        let (upstream_url, received) = mock_json_upstream(json!({
            "id": "msg-test",
            "type": "message",
            "role": "assistant",
            "content": [{ "type": "text", "text": "2" }],
            "usage": {
                "input_tokens": 3,
                "output_tokens": 1,
                "cache_creation_input_tokens": 0,
                "cache_read_input_tokens": 2
            }
        }));
        let db = Arc::new(Database::memory().unwrap());
        let gateway = GatewayRuntime::new();
        let port = free_port();
        db.set_gateway_port(port).unwrap();
        let key = db.regenerate_local_api_key().unwrap();
        db.create_provider(anthropic_provider("Mock Claude", &upstream_url))
            .unwrap();

        gateway.start(db.clone()).unwrap();
        let response: Value = reqwest::blocking::Client::new()
            .post(format!("http://127.0.0.1:{port}/v1/messages"))
            .header("x-api-key", key)
            .json(&json!({
                "model": "claude-sonnet",
                "messages": [{ "role": "user", "content": "1+1=?" }],
                "max_tokens": 32,
                "stream": false
            }))
            .send()
            .unwrap()
            .json()
            .unwrap();
        gateway.stop().unwrap();

        let upstream_request = received.recv().unwrap();
        assert!(upstream_request.starts_with("POST /v1/messages "));
        assert!(upstream_request.contains(r#""content":"1+1=?""#));
        assert_eq!(response["type"], "message");
        assert_eq!(response["content"][0]["text"], "2");

        let logs = db.get_gateway_logs(10).unwrap();
        assert_eq!(logs[0].resolved_provider.as_deref(), Some("Mock Claude"));
        assert_eq!(logs[0].protocol_in, "anthropic");
        assert_eq!(logs[0].protocol_out.as_deref(), Some("anthropic"));
        assert_eq!(logs[0].input_tokens, Some(3));
        assert_eq!(logs[0].output_tokens, Some(1));
        assert_eq!(logs[0].total_tokens, Some(4));
        assert_eq!(logs[0].cache_read_input_tokens, Some(2));
    }

    #[test]
    fn proxies_openai_chat_to_anthropic_provider_without_conversion() {
        let (upstream_url, received) = mock_json_upstream(json!({
            "id": "chatcmpl-test",
            "object": "chat.completion",
            "choices": [{ "message": { "role": "assistant", "content": "2" } }],
            "usage": { "input_tokens": 3, "output_tokens": 1 }
        }));
        let db = Arc::new(Database::memory().unwrap());
        let gateway = GatewayRuntime::new();
        let port = free_port();
        db.set_gateway_port(port).unwrap();
        let key = db.regenerate_local_api_key().unwrap();
        db.create_provider(anthropic_provider("Mock Claude", &upstream_url))
            .unwrap();

        gateway.start(db.clone()).unwrap();
        let response: Value = reqwest::blocking::Client::new()
            .post(format!("http://127.0.0.1:{port}/v1/chat/completions"))
            .bearer_auth(key)
            .json(&json!({
                "model": "claude-sonnet",
                "messages": [
                    { "role": "system", "content": "Be exact." },
                    { "role": "user", "content": "1+1=?" }
                ],
                "max_tokens": 32,
                "stream": false
            }))
            .send()
            .unwrap()
            .json()
            .unwrap();
        gateway.stop().unwrap();

        let upstream_request = received.recv().unwrap();
        assert!(upstream_request.starts_with("POST /v1/chat/completions "));
        assert!(upstream_request.contains(r#""role":"system""#));
        assert!(upstream_request.contains(r#""content":"1+1=?""#));
        assert_eq!(response["object"], "chat.completion");
        assert_eq!(response["choices"][0]["message"]["content"], "2");

        let logs = db.get_gateway_logs(10).unwrap();
        assert_eq!(logs[0].protocol_in, "openai");
        assert_eq!(logs[0].protocol_out.as_deref(), Some("anthropic"));
        assert_eq!(logs[0].input_tokens, Some(3));
        assert_eq!(logs[0].output_tokens, Some(1));
        assert_eq!(logs[0].total_tokens, Some(4));
    }

    #[test]
    fn proxies_openai_responses_to_responses_upstream() {
        let (upstream_url, received) = mock_json_upstream(json!({
            "id": "resp-test",
            "object": "response",
            "output_text": "2",
            "usage": { "input_tokens": 3, "output_tokens": 1, "total_tokens": 4 }
        }));
        let db = Arc::new(Database::memory().unwrap());
        let gateway = GatewayRuntime::new();
        let port = free_port();
        db.set_gateway_port(port).unwrap();
        let key = db.regenerate_local_api_key().unwrap();
        db.create_provider(openai_provider("Mock OpenAI", &upstream_url))
            .unwrap();

        gateway.start(db.clone()).unwrap();
        let response: Value = reqwest::blocking::Client::new()
            .post(format!("http://127.0.0.1:{port}/v1/responses"))
            .bearer_auth(key)
            .json(&json!({
                "model": "llama3",
                "input": "1+1=?",
                "max_output_tokens": 16
            }))
            .send()
            .unwrap()
            .json()
            .unwrap();
        gateway.stop().unwrap();

        let upstream_request = received.recv().unwrap();
        assert!(upstream_request.starts_with("POST /v1/responses "));
        assert!(upstream_request.contains(r#""input":"1+1=?""#));
        assert_eq!(response["object"], "response");
        assert_eq!(response["output_text"], "2");
        assert_eq!(response["usage"]["total_tokens"], 4);
        let logs = db.get_gateway_logs(10).unwrap();
        assert_eq!(logs[0].endpoint, "/v1/responses");
        assert_eq!(logs[0].protocol_in, "openai");
        assert_eq!(logs[0].protocol_out.as_deref(), Some("openai"));
        assert_eq!(logs[0].total_tokens, Some(4));
    }

    #[test]
    fn proxies_openai_responses_stream_without_conversion() {
        let (upstream_url, received) = mock_sse_upstream(vec![
            json!({
                "type": "response.output_text.delta",
                "delta": "2"
            }),
            json!({
                "type": "response.completed",
                "response": { "output_text": "2" }
            }),
        ]);
        let db = Arc::new(Database::memory().unwrap());
        let gateway = GatewayRuntime::new();
        let port = free_port();
        db.set_gateway_port(port).unwrap();
        let key = db.regenerate_local_api_key().unwrap();
        db.create_provider(openai_provider("Mock OpenAI", &upstream_url))
            .unwrap();

        gateway.start(db.clone()).unwrap();
        let response = reqwest::blocking::Client::new()
            .post(format!("http://127.0.0.1:{port}/v1/responses"))
            .bearer_auth(key)
            .json(&json!({
                "model": "llama3",
                "input": "1+1=?",
                "max_output_tokens": 16,
                "stream": true
            }))
            .send()
            .unwrap();
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let body = response.text().unwrap();
        gateway.stop().unwrap();

        let upstream_request = received.recv().unwrap();
        assert_eq!(status, 200);
        assert!(content_type.starts_with("text/event-stream"));
        assert!(upstream_request.starts_with("POST /v1/responses "));
        assert!(upstream_request.contains(r#""stream":true"#));
        assert!(!upstream_request.contains(r#""include_usage":true"#));
        assert!(body.contains(r#""delta":"2""#));
        assert!(body.contains(r#""output_text":"2""#));

        let logs = db.get_gateway_logs(10).unwrap();
        assert_eq!(logs[0].endpoint, "/v1/responses");
        assert_eq!(logs[0].protocol_in, "openai");
        assert_eq!(logs[0].protocol_out.as_deref(), Some("openai"));
        assert!(logs[0].stream);
    }

    #[test]
    #[ignore = "requires LAG_REAL_OPENAI_BASE_URL and LAG_REAL_OPENAI_MODEL; optional LAG_REAL_OPENAI_API_KEY"]
    fn real_openai_compatible_provider_routes_through_local_gateway() {
        let base_url = std::env::var("LAG_REAL_OPENAI_BASE_URL")
            .expect("Set LAG_REAL_OPENAI_BASE_URL, for example http://127.0.0.1:11434/v1");
        let model = std::env::var("LAG_REAL_OPENAI_MODEL")
            .expect("Set LAG_REAL_OPENAI_MODEL, for example llama3.2");
        let api_key = std::env::var("LAG_REAL_OPENAI_API_KEY").ok();
        let expected_text =
            std::env::var("LAG_REAL_EXPECT_TEXT").unwrap_or_else(|_| "2".to_string());

        let db = Arc::new(Database::memory().unwrap());
        let gateway = GatewayRuntime::new();
        let port = free_port();
        db.set_gateway_port(port).unwrap();
        let local_key = db.regenerate_local_api_key().unwrap();
        db.create_provider(ProviderInput {
            name: "Real OpenAI-compatible".to_string(),
            base_url,
            protocol: ProviderProtocol::OpenAiCompatible,
            enabled: true,
            default_model: model.clone(),
            models: vec![model.clone()],
            model_prefixes: Vec::new(),
            timeout_ms: 120_000,
            max_tokens_field: "max_tokens".to_string(),
            api_key,
        })
        .unwrap();

        gateway.start(db.clone()).unwrap();
        let response = reqwest::blocking::Client::new()
            .post(format!("http://127.0.0.1:{port}/v1/chat/completions"))
            .bearer_auth(local_key)
            .json(&json!({
                "model": model,
                "messages": [
                    {
                        "role": "user",
                        "content": "1+1=?"
                    }
                ],
                "max_tokens": 64,
                "stream": false
            }))
            .send()
            .unwrap();
        let status = response.status().as_u16();
        let body: Value = response.json().unwrap();
        gateway.stop().unwrap();

        assert!(
            status < 300,
            "real provider request failed with status {status}: {body}"
        );
        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default();
        assert!(
            !content.trim().is_empty(),
            "real provider returned empty assistant content: {body}"
        );
        assert!(
            content.contains(&expected_text),
            "assistant content did not contain expected text {expected_text:?}: {content:?}"
        );

        let logs = db.get_gateway_logs(10).unwrap();
        assert_eq!(
            logs[0].resolved_provider.as_deref(),
            Some("Real OpenAI-compatible")
        );
        assert_eq!(logs[0].protocol_in, "openai");
        assert_eq!(logs[0].protocol_out.as_deref(), Some("openai"));
        assert_eq!(logs[0].status_code, status as i64);
        assert!(
            logs[0].total_tokens.unwrap_or_default() > 0,
            "real provider did not return usable token usage: {:?}",
            logs[0]
        );
    }

    #[test]
    #[ignore = "requires LAG_REAL_OPENAI_BASE_URL and LAG_REAL_OPENAI_MODEL; optional LAG_REAL_OPENAI_API_KEY"]
    fn real_openai_responses_api_routes_through_local_gateway() {
        let base_url = std::env::var("LAG_REAL_OPENAI_BASE_URL")
            .expect("Set LAG_REAL_OPENAI_BASE_URL, for example http://127.0.0.1:11434/v1");
        let model = std::env::var("LAG_REAL_OPENAI_MODEL")
            .expect("Set LAG_REAL_OPENAI_MODEL, for example llama3.2");
        let api_key = std::env::var("LAG_REAL_OPENAI_API_KEY").ok();
        let expected_text =
            std::env::var("LAG_REAL_EXPECT_TEXT").unwrap_or_else(|_| "2".to_string());

        let db = Arc::new(Database::memory().unwrap());
        let gateway = GatewayRuntime::new();
        let port = free_port();
        db.set_gateway_port(port).unwrap();
        let local_key = db.regenerate_local_api_key().unwrap();
        db.create_provider(ProviderInput {
            name: "Real OpenAI-compatible".to_string(),
            base_url,
            protocol: ProviderProtocol::OpenAiCompatible,
            enabled: true,
            default_model: model.clone(),
            models: vec![model.clone()],
            model_prefixes: Vec::new(),
            timeout_ms: 120_000,
            max_tokens_field: "max_tokens".to_string(),
            api_key,
        })
        .unwrap();

        gateway.start(db.clone()).unwrap();
        let response = reqwest::blocking::Client::new()
            .post(format!("http://127.0.0.1:{port}/v1/responses"))
            .bearer_auth(local_key)
            .json(&json!({
                "model": model,
                "input": [{
                    "role": "user",
                    "content": [{ "type": "input_text", "text": "1+1=?" }]
                }],
                "max_output_tokens": 64,
                "stream": false
            }))
            .send()
            .unwrap();
        let status = response.status().as_u16();
        let body: Value = response.json().unwrap();
        gateway.stop().unwrap();

        assert!(
            status < 300,
            "real responses request failed with status {status}: {body}"
        );
        let content = response_text_from_responses_body(&body);
        assert!(
            content.contains(&expected_text),
            "responses body did not contain expected text {expected_text:?}: {body}"
        );

        let logs = db.get_gateway_logs(10).unwrap();
        assert_eq!(logs[0].endpoint, "/v1/responses");
        assert_eq!(logs[0].protocol_in, "openai");
        assert_eq!(logs[0].protocol_out.as_deref(), Some("openai"));
        assert!(
            logs[0].total_tokens.unwrap_or_default() > 0,
            "real responses provider did not return usable token usage: {:?}",
            logs[0]
        );
    }

    #[test]
    #[ignore = "requires LAG_REAL_OPENAI_BASE_URL and LAG_REAL_OPENAI_MODEL; optional LAG_REAL_OPENAI_API_KEY"]
    fn real_openai_responses_stream_routes_through_local_gateway() {
        let base_url = std::env::var("LAG_REAL_OPENAI_BASE_URL")
            .expect("Set LAG_REAL_OPENAI_BASE_URL, for example http://127.0.0.1:11434/v1");
        let model = std::env::var("LAG_REAL_OPENAI_MODEL")
            .expect("Set LAG_REAL_OPENAI_MODEL, for example llama3.2");
        let api_key = std::env::var("LAG_REAL_OPENAI_API_KEY").ok();
        let expected_text =
            std::env::var("LAG_REAL_EXPECT_TEXT").unwrap_or_else(|_| "2".to_string());

        let db = Arc::new(Database::memory().unwrap());
        let gateway = GatewayRuntime::new();
        let port = free_port();
        db.set_gateway_port(port).unwrap();
        let local_key = db.regenerate_local_api_key().unwrap();
        db.create_provider(ProviderInput {
            name: "Real OpenAI-compatible".to_string(),
            base_url,
            protocol: ProviderProtocol::OpenAiCompatible,
            enabled: true,
            default_model: model.clone(),
            models: vec![model.clone()],
            model_prefixes: Vec::new(),
            timeout_ms: 120_000,
            max_tokens_field: "max_tokens".to_string(),
            api_key,
        })
        .unwrap();

        gateway.start(db.clone()).unwrap();
        let response = reqwest::blocking::Client::new()
            .post(format!("http://127.0.0.1:{port}/v1/responses"))
            .bearer_auth(local_key)
            .json(&json!({
                "model": model,
                "input": [{
                    "role": "user",
                    "content": [{ "type": "input_text", "text": "1+1=?" }]
                }],
                "max_output_tokens": 64,
                "stream": true
            }))
            .send()
            .unwrap();
        let status = response.status().as_u16();
        let body = response.text().unwrap();
        gateway.stop().unwrap();

        assert!(
            status < 300,
            "real responses stream request failed with status {status}: {body}"
        );
        assert!(
            body.contains("response.output_text.delta"),
            "responses stream did not emit text delta events: {body}"
        );
        assert!(
            body.contains(&expected_text),
            "responses stream did not contain expected text {expected_text:?}: {body}"
        );
        assert!(
            body.contains("response.completed"),
            "responses stream did not complete: {body}"
        );

        let logs = db.get_gateway_logs(10).unwrap();
        assert_eq!(logs[0].endpoint, "/v1/responses");
        assert_eq!(logs[0].protocol_in, "openai");
        assert_eq!(logs[0].protocol_out.as_deref(), Some("openai"));
        assert!(logs[0].stream);
    }

    fn openai_provider(name: &str, base_url: &str) -> ProviderInput {
        ProviderInput {
            name: name.to_string(),
            base_url: base_url.to_string(),
            protocol: ProviderProtocol::OpenAiCompatible,
            enabled: true,
            default_model: "llama3".to_string(),
            models: vec!["llama3".to_string()],
            model_prefixes: vec!["llama".to_string()],
            timeout_ms: 5000,
            max_tokens_field: "max_tokens".to_string(),
            api_key: None,
        }
    }

    fn anthropic_provider(name: &str, base_url: &str) -> ProviderInput {
        ProviderInput {
            name: name.to_string(),
            base_url: base_url.to_string(),
            protocol: ProviderProtocol::AnthropicCompatible,
            enabled: true,
            default_model: "claude-sonnet".to_string(),
            models: vec!["claude-sonnet".to_string()],
            model_prefixes: vec!["claude".to_string()],
            timeout_ms: 5000,
            max_tokens_field: "max_tokens".to_string(),
            api_key: None,
        }
    }

    fn response_text_from_responses_body(body: &Value) -> String {
        if let Some(text) = body.get("output_text").and_then(Value::as_str) {
            return text.to_string();
        }
        body.get("output")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .flat_map(|item| {
                item.get("content")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
            })
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("")
    }

    fn free_port() -> u16 {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.local_addr().unwrap().port()
    }

    fn mock_json_upstream(response: Value) -> (String, mpsc::Receiver<String>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request = String::new();
                let mut first_line = String::new();
                reader.read_line(&mut first_line).unwrap();
                request.push_str(&first_line);
                let mut content_length = 0_usize;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                    if let Some((key, value)) = line.split_once(':') {
                        if key.eq_ignore_ascii_case("content-length") {
                            content_length = value.trim().parse().unwrap_or(0);
                        }
                    }
                }
                let mut body = vec![0_u8; content_length];
                if content_length > 0 {
                    reader.read_exact(&mut body).unwrap();
                    request.push_str(&String::from_utf8_lossy(&body));
                }
                tx.send(request).unwrap();
                let bytes = serde_json::to_vec(&response).unwrap();
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    bytes.len()
                )
                .unwrap();
                stream.write_all(&bytes).unwrap();
            }
        });
        (format!("http://127.0.0.1:{port}/v1"), rx)
    }

    fn mock_sse_upstream(chunks: Vec<Value>) -> (String, mpsc::Receiver<String>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request = String::new();
                let mut first_line = String::new();
                reader.read_line(&mut first_line).unwrap();
                request.push_str(&first_line);
                let mut content_length = 0_usize;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                    if let Some((key, value)) = line.split_once(':') {
                        if key.eq_ignore_ascii_case("content-length") {
                            content_length = value.trim().parse().unwrap_or(0);
                        }
                    }
                }
                let mut body = vec![0_u8; content_length];
                if content_length > 0 {
                    reader.read_exact(&mut body).unwrap();
                    request.push_str(&String::from_utf8_lossy(&body));
                }
                tx.send(request).unwrap();
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n"
                )
                .unwrap();
                for chunk in chunks {
                    writeln!(stream, "data: {}\n", serde_json::to_string(&chunk).unwrap()).unwrap();
                }
                write!(stream, "data: [DONE]\n\n").unwrap();
            }
        });
        (format!("http://127.0.0.1:{port}/v1"), rx)
    }
}

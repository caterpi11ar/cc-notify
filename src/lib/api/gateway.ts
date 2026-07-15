import { invoke } from "@tauri-apps/api/core";

export type ProviderProtocol =
  | "openai_compatible"
  | "anthropic_compatible"
  | "codex_experimental";

export interface GatewayStatus {
  running: boolean;
  host: string;
  port: number;
  openai_base_url: string;
  anthropic_base_url: string;
}

export interface ApiKeyStatus {
  exists: boolean;
  preview: string | null;
}

export interface Provider {
  id: string;
  name: string;
  base_url: string;
  protocol: ProviderProtocol;
  enabled: boolean;
  default_model: string;
  models: string[];
  model_prefixes: string[];
  timeout_ms: number;
  max_tokens_field: string;
  secret_ref: string | null;
  has_api_key: boolean;
  health_status: string;
  created_at: number;
  updated_at: number;
}

export interface GatewayLog {
  id: number;
  time: number;
  client_type: string;
  endpoint: string;
  requested_model: string;
  resolved_provider: string | null;
  resolved_model: string | null;
  protocol_in: string;
  protocol_out: string | null;
  status_code: number;
  latency_ms: number;
  stream: boolean;
  token_estimate: number | null;
  input_tokens: number | null;
  output_tokens: number | null;
  total_tokens: number | null;
  cache_creation_input_tokens: number | null;
  cache_read_input_tokens: number | null;
  fidelity_mode: string;
  error_message: string | null;
}

export const gatewayApi = {
  getStatus: () => invoke<GatewayStatus>("get_gateway_status"),
  start: () => invoke<GatewayStatus>("start_gateway"),
  stop: () => invoke<void>("stop_gateway"),
  restart: () => invoke<GatewayStatus>("restart_gateway"),
  getApiKeyStatus: () => invoke<ApiKeyStatus>("get_local_api_key_status"),
  regenerateApiKey: () => invoke<string>("regenerate_local_api_key"),
  getApiKeyOnce: () => invoke<string | null>("get_local_api_key_once"),
  getProviders: () => invoke<Provider[]>("get_providers"),
  getLogs: (limit = 100) => invoke<GatewayLog[]>("get_gateway_logs", { limit }),
  clearLogs: () => invoke<void>("clear_gateway_logs"),
};

import { http, HttpResponse } from "msw";

const TAURI_ENDPOINT = "http://tauri.local";

const success = <T>(payload: T) => HttpResponse.json(payload as never);

export const handlers = [
  http.post(`${TAURI_ENDPOINT}/get_gateway_status`, () =>
    success({
      running: false,
      host: "127.0.0.1",
      port: 17777,
      openai_base_url: "http://127.0.0.1:17777/v1",
      anthropic_base_url: "http://127.0.0.1:17777",
    }),
  ),
  http.post(`${TAURI_ENDPOINT}/start_gateway`, () =>
    success({
      running: true,
      host: "127.0.0.1",
      port: 17777,
      openai_base_url: "http://127.0.0.1:17777/v1",
      anthropic_base_url: "http://127.0.0.1:17777",
    }),
  ),
  http.post(`${TAURI_ENDPOINT}/stop_gateway`, () => success(null)),
  http.post(`${TAURI_ENDPOINT}/restart_gateway`, () =>
    success({
      running: true,
      host: "127.0.0.1",
      port: 17777,
      openai_base_url: "http://127.0.0.1:17777/v1",
      anthropic_base_url: "http://127.0.0.1:17777",
    }),
  ),
  http.post(`${TAURI_ENDPOINT}/get_local_api_key_status`, () =>
    success({ exists: true, preview: "lgw_test...1234" }),
  ),
  http.post(`${TAURI_ENDPOINT}/get_local_api_key_once`, () =>
    success("lgw_test_key"),
  ),
  http.post(`${TAURI_ENDPOINT}/regenerate_local_api_key`, () =>
    success("lgw_new_key"),
  ),
  http.post(`${TAURI_ENDPOINT}/get_providers`, () =>
    success([
      {
        id: "provider-1",
        name: "Local OpenAI",
        base_url: "http://127.0.0.1:11434/v1",
        protocol: "openai_compatible",
        enabled: true,
        default_model: "llama3",
        models: ["llama3"],
        model_prefixes: ["local-"],
        timeout_ms: 60000,
        max_tokens_field: "max_tokens",
        secret_ref: "provider:provider-1:api_key",
        has_api_key: true,
        health_status: "ok",
        created_at: 1,
        updated_at: 1,
      },
    ]),
  ),
  http.post(`${TAURI_ENDPOINT}/get_gateway_logs`, () =>
    success([
      {
        id: 1,
        time: 1,
        client_type: "openai",
        endpoint: "/v1/chat/completions",
        requested_model: "local-code",
        resolved_provider: "Local OpenAI",
        resolved_model: "llama3",
        protocol_in: "openai",
        protocol_out: "openai",
        status_code: 200,
        latency_ms: 24,
        stream: false,
        token_estimate: 8,
        input_tokens: 4,
        output_tokens: 2,
        total_tokens: 6,
        cache_creation_input_tokens: 0,
        cache_read_input_tokens: 0,
        fidelity_mode: "strict",
        error_message: null,
      },
    ]),
  ),
  http.post(`${TAURI_ENDPOINT}/clear_gateway_logs`, () => success(null)),
];

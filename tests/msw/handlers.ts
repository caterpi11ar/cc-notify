import { http, HttpResponse } from "msw";
import type { Channel, EventType, Rule, Routing, Template } from "@/types";
import {
  getChannels,
  createChannel,
  updateChannel,
  deleteChannel,
  testChannel,
  getEventTypes,
  createEventType,
  updateEventType,
  deleteEventType,
  getRules,
  createRule,
  updateRule,
  deleteRule,
  getRoutings,
  setRouting,
  deleteRouting,
  getRoutingsByEventType,
  getTemplates,
  createTemplate,
  updateTemplate,
  deleteTemplate,
  getHistory,
  getHistoryByEventType,
  clearHistory,
  getSettings,
  getSetting,
  setSetting,
  deleteSetting,
  getHooksStatus,
  installHook,
  uninstallHook,
} from "./state";

const TAURI_ENDPOINT = "http://tauri.local";

const withJson = async <T>(request: Request): Promise<T> => {
  try {
    const body = await request.text();
    if (!body) return {} as T;
    return JSON.parse(body) as T;
  } catch {
    return {} as T;
  }
};

const success = <T>(payload: T) => HttpResponse.json(payload as never);

export const handlers = [
  // ── Gateway ──
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
  // ── Channels ──
  http.post(`${TAURI_ENDPOINT}/get_channels`, () => success(getChannels())),

  http.post(`${TAURI_ENDPOINT}/create_channel`, async ({ request }) => {
    const { channel } = await withJson<{
      channel: Omit<Channel, "id" | "created_at" | "updated_at">;
    }>(request);
    return success(createChannel(channel));
  }),

  http.post(`${TAURI_ENDPOINT}/update_channel`, async ({ request }) => {
    const { id, channel } = await withJson<{
      id: string;
      channel: Partial<Channel>;
    }>(request);
    updateChannel(id, channel);
    return success(null);
  }),

  http.post(`${TAURI_ENDPOINT}/delete_channel`, async ({ request }) => {
    const { id } = await withJson<{ id: string }>(request);
    deleteChannel(id);
    return success(null);
  }),

  http.post(`${TAURI_ENDPOINT}/test_channel`, async ({ request }) => {
    const { id } = await withJson<{ id: string }>(request);
    return success(testChannel(id));
  }),

  // ── Event Types ──
  http.post(`${TAURI_ENDPOINT}/get_event_types`, () =>
    success(getEventTypes()),
  ),

  http.post(`${TAURI_ENDPOINT}/create_event_type`, async ({ request }) => {
    const { eventType } = await withJson<{
      eventType: Omit<EventType, "id">;
    }>(request);
    return success(createEventType(eventType));
  }),

  http.post(`${TAURI_ENDPOINT}/update_event_type`, async ({ request }) => {
    const { id, eventType } = await withJson<{
      id: string;
      eventType: Partial<EventType>;
    }>(request);
    updateEventType(id, eventType);
    return success(null);
  }),

  http.post(`${TAURI_ENDPOINT}/delete_event_type`, async ({ request }) => {
    const { id } = await withJson<{ id: string }>(request);
    deleteEventType(id);
    return success(null);
  }),

  // ── Rules ──
  http.post(`${TAURI_ENDPOINT}/get_rules`, () => success(getRules())),

  http.post(`${TAURI_ENDPOINT}/create_rule`, async ({ request }) => {
    const { rule } = await withJson<{
      rule: Omit<Rule, "id" | "created_at">;
    }>(request);
    return success(createRule(rule));
  }),

  http.post(`${TAURI_ENDPOINT}/update_rule`, async ({ request }) => {
    const { id, rule } = await withJson<{
      id: string;
      rule: Partial<Rule>;
    }>(request);
    updateRule(id, rule);
    return success(null);
  }),

  http.post(`${TAURI_ENDPOINT}/delete_rule`, async ({ request }) => {
    const { id } = await withJson<{ id: string }>(request);
    deleteRule(id);
    return success(null);
  }),

  // ── Routing ──
  http.post(`${TAURI_ENDPOINT}/get_routings`, () => success(getRoutings())),

  http.post(`${TAURI_ENDPOINT}/set_routing`, async ({ request }) => {
    const { routing } = await withJson<{ routing: Routing }>(request);
    setRouting(routing);
    return success(null);
  }),

  http.post(`${TAURI_ENDPOINT}/delete_routing`, async ({ request }) => {
    const { eventTypeId, channelId } = await withJson<{
      eventTypeId: string;
      channelId: string;
    }>(request);
    deleteRouting(eventTypeId, channelId);
    return success(null);
  }),

  http.post(
    `${TAURI_ENDPOINT}/get_routings_by_event_type`,
    async ({ request }) => {
      const { eventTypeId } = await withJson<{ eventTypeId: string }>(request);
      return success(getRoutingsByEventType(eventTypeId));
    },
  ),

  // ── Templates ──
  http.post(`${TAURI_ENDPOINT}/get_templates`, () => success(getTemplates())),

  http.post(`${TAURI_ENDPOINT}/create_template`, async ({ request }) => {
    const { template } = await withJson<{
      template: Omit<Template, "id">;
    }>(request);
    return success(createTemplate(template));
  }),

  http.post(`${TAURI_ENDPOINT}/update_template`, async ({ request }) => {
    const { id, template } = await withJson<{
      id: string;
      template: Partial<Template>;
    }>(request);
    updateTemplate(id, template);
    return success(null);
  }),

  http.post(`${TAURI_ENDPOINT}/delete_template`, async ({ request }) => {
    const { id } = await withJson<{ id: string }>(request);
    deleteTemplate(id);
    return success(null);
  }),

  // ── History ──
  http.post(`${TAURI_ENDPOINT}/get_history`, async ({ request }) => {
    const { limit, offset } = await withJson<{
      limit?: number;
      offset?: number;
    }>(request);
    return success(getHistory(limit, offset));
  }),

  http.post(
    `${TAURI_ENDPOINT}/get_history_by_event_type`,
    async ({ request }) => {
      const { eventTypeId } = await withJson<{ eventTypeId: string }>(request);
      return success(getHistoryByEventType(eventTypeId));
    },
  ),

  http.post(`${TAURI_ENDPOINT}/clear_history`, () => {
    clearHistory();
    return success(null);
  }),

  // ── Settings ──
  http.post(`${TAURI_ENDPOINT}/get_settings`, () => success(getSettings())),

  http.post(`${TAURI_ENDPOINT}/get_setting`, async ({ request }) => {
    const { key } = await withJson<{ key: string }>(request);
    return success(getSetting(key));
  }),

  http.post(`${TAURI_ENDPOINT}/set_setting`, async ({ request }) => {
    const { key, value } = await withJson<{ key: string; value: string }>(
      request,
    );
    setSetting(key, value);
    return success(null);
  }),

  http.post(`${TAURI_ENDPOINT}/delete_setting`, async ({ request }) => {
    const { key } = await withJson<{ key: string }>(request);
    deleteSetting(key);
    return success(null);
  }),

  // ── Hooks ──
  http.post(`${TAURI_ENDPOINT}/get_hooks_status`, () =>
    success(getHooksStatus()),
  ),

  http.post(`${TAURI_ENDPOINT}/install_hook`, async ({ request }) => {
    const { tool } = await withJson<{ tool: string }>(request);
    installHook(tool);
    return success(null);
  }),

  http.post(`${TAURI_ENDPOINT}/uninstall_hook`, async ({ request }) => {
    const { tool } = await withJson<{ tool: string }>(request);
    uninstallHook(tool);
    return success(null);
  }),
];

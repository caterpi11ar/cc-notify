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

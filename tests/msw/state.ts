import type {
  Channel,
  EventType,
  Rule,
  Routing,
  Template,
  NotificationHistory,
  Settings,
  HooksStatus,
} from "@/types";

// ── Default fixture data ──

const createDefaultChannels = (): Channel[] => [
  {
    id: "ch-1",
    name: "Native Notification",
    channel_type: "native",
    config: { timeout: 10 },
    enabled: true,
    sort_index: 0,
    created_at: 1700000000,
    updated_at: 1700000000,
  },
  {
    id: "ch-2",
    name: "Slack Alerts",
    channel_type: "slack",
    config: { webhook_url: "https://hooks.slack.com/test", channel: "#dev" },
    enabled: true,
    sort_index: 1,
    created_at: 1700000001,
    updated_at: 1700000001,
  },
  {
    id: "ch-3",
    name: "Discord Logs",
    channel_type: "discord",
    config: { webhook_url: "https://discord.com/api/webhooks/test" },
    enabled: false,
    sort_index: 2,
    created_at: 1700000002,
    updated_at: 1700000002,
  },
];

const createDefaultEventTypes = (): EventType[] => [
  {
    id: "evt-1",
    name: "Task Complete",
    category: "claude_hook",
    is_builtin: true,
    config: {},
    enabled: true,
  },
  {
    id: "evt-2",
    name: "Error Detected",
    category: "claude_hook",
    is_builtin: true,
    config: {},
    enabled: true,
  },
  {
    id: "evt-3",
    name: "Long Running Task",
    category: "extended",
    is_builtin: false,
    config: {},
    enabled: true,
  },
  {
    id: "evt-4",
    name: "Custom Alert",
    category: "custom",
    is_builtin: false,
    config: {},
    enabled: false,
  },
];

const createDefaultRules = (): Rule[] => [
  {
    id: "rule-1",
    name: "Error Keyword",
    rule_type: "keyword",
    pattern: "error",
    event_type_id: "evt-2",
    enabled: true,
    created_at: 1700000000,
  },
  {
    id: "rule-2",
    name: "Source File Change",
    rule_type: "file_change",
    pattern: "src/**/*.ts",
    event_type_id: "evt-3",
    enabled: true,
    created_at: 1700000001,
  },
];

const createDefaultRoutings = (): Routing[] => [
  { event_type_id: "evt-1", channel_id: "ch-1", enabled: true, priority: 0 },
  { event_type_id: "evt-1", channel_id: "ch-2", enabled: true, priority: 1 },
  { event_type_id: "evt-2", channel_id: "ch-2", enabled: true, priority: 0 },
];

const createDefaultTemplates = (): Template[] => [
  {
    id: "tpl-1",
    name: "Default Slack",
    channel_type: "slack",
    body_template: "{{event_name}}: {{message}}",
    format: "text",
    is_default: true,
  },
];

const createDefaultHistory = (): NotificationHistory[] => [
  {
    id: 1,
    event_type_id: "evt-1",
    channel_id: "ch-1",
    status: "sent",
    message_body: "Task completed successfully",
    error_message: null,
    metadata: {},
    created_at: 1700001000,
  },
  {
    id: 2,
    event_type_id: "evt-2",
    channel_id: "ch-2",
    status: "failed",
    message_body: "Error notification",
    error_message: "Connection timeout",
    metadata: {},
    created_at: 1700002000,
  },
  {
    id: 3,
    event_type_id: "evt-1",
    channel_id: "ch-2",
    status: "sent",
    message_body: "Task done",
    error_message: null,
    metadata: {},
    created_at: 1700003000,
  },
];

const createDefaultSettings = (): Settings => ({
  language: "en",
  history_retention_days: "30",
  quiet_hours_enabled: "false",
  quiet_hours_start: "22:00",
  quiet_hours_end: "08:00",
  quiet_hours_days: '["mon","tue","wed","thu","fri"]',
  rate_limit_max_per_minute: "10",
  rate_limit_cooldown_seconds: "5",
  kill_switch: "false",
  sound_enabled: "true",
  sound_volume: "80",
  voice_enabled: "false",
  voice_name: "",
});

const createDefaultHooksStatus = (): HooksStatus => ({
  claude: true,
  codex: false,
  gemini: false,
});

// ── Mutable state ──

let channels = createDefaultChannels();
let eventTypes = createDefaultEventTypes();
let rules = createDefaultRules();
let routings = createDefaultRoutings();
let templates = createDefaultTemplates();
let history = createDefaultHistory();
let settings = createDefaultSettings();
let hooksStatus = createDefaultHooksStatus();

let nextChannelId = 100;
let nextEventTypeId = 100;
let nextRuleId = 100;
let nextTemplateId = 100;
let nextHistoryId = 100;

// ── Reset ──

export const resetState = () => {
  channels = createDefaultChannels();
  eventTypes = createDefaultEventTypes();
  rules = createDefaultRules();
  routings = createDefaultRoutings();
  templates = createDefaultTemplates();
  history = createDefaultHistory();
  settings = createDefaultSettings();
  hooksStatus = createDefaultHooksStatus();
  nextChannelId = 100;
  nextEventTypeId = 100;
  nextRuleId = 100;
  nextTemplateId = 100;
  nextHistoryId = 100;
};

// ── Channels ──

export const getChannels = (): Channel[] =>
  JSON.parse(JSON.stringify(channels));

export const createChannel = (
  data: Omit<Channel, "id" | "created_at" | "updated_at">,
): Channel => {
  const now = Date.now();
  const ch: Channel = {
    ...data,
    id: `ch-${nextChannelId++}`,
    created_at: now,
    updated_at: now,
  };
  channels.push(ch);
  return JSON.parse(JSON.stringify(ch));
};

export const updateChannel = (id: string, data: Partial<Channel>): void => {
  const idx = channels.findIndex((c) => c.id === id);
  if (idx === -1) throw new Error(`Channel ${id} not found`);
  channels[idx] = { ...channels[idx], ...data, updated_at: Date.now() };
};

export const deleteChannel = (id: string): void => {
  channels = channels.filter((c) => c.id !== id);
};

export const testChannel = (
  _id: string,
): { success: boolean; message?: string } => {
  return { success: true, message: "Test notification sent" };
};

// ── Event Types ──

export const getEventTypes = (): EventType[] =>
  JSON.parse(JSON.stringify(eventTypes));

export const createEventType = (data: Omit<EventType, "id">): EventType => {
  const evt: EventType = { ...data, id: `evt-${nextEventTypeId++}` };
  eventTypes.push(evt);
  return JSON.parse(JSON.stringify(evt));
};

export const updateEventType = (
  id: string,
  data: Partial<EventType>,
): void => {
  const idx = eventTypes.findIndex((e) => e.id === id);
  if (idx === -1) throw new Error(`EventType ${id} not found`);
  eventTypes[idx] = { ...eventTypes[idx], ...data };
};

export const deleteEventType = (id: string): void => {
  eventTypes = eventTypes.filter((e) => e.id !== id);
};

// ── Rules ──

export const getRules = (): Rule[] => JSON.parse(JSON.stringify(rules));

export const createRule = (data: Omit<Rule, "id" | "created_at">): Rule => {
  const r: Rule = {
    ...data,
    id: `rule-${nextRuleId++}`,
    created_at: Date.now(),
  };
  rules.push(r);
  return JSON.parse(JSON.stringify(r));
};

export const updateRule = (id: string, data: Partial<Rule>): void => {
  const idx = rules.findIndex((r) => r.id === id);
  if (idx === -1) throw new Error(`Rule ${id} not found`);
  rules[idx] = { ...rules[idx], ...data };
};

export const deleteRule = (id: string): void => {
  rules = rules.filter((r) => r.id !== id);
};

// ── Routing ──

export const getRoutings = (): Routing[] =>
  JSON.parse(JSON.stringify(routings));

export const setRouting = (routing: Routing): void => {
  const idx = routings.findIndex(
    (r) =>
      r.event_type_id === routing.event_type_id &&
      r.channel_id === routing.channel_id,
  );
  if (idx >= 0) {
    routings[idx] = routing;
  } else {
    routings.push(routing);
  }
};

export const deleteRouting = (
  eventTypeId: string,
  channelId: string,
): void => {
  routings = routings.filter(
    (r) => !(r.event_type_id === eventTypeId && r.channel_id === channelId),
  );
};

export const getRoutingsByEventType = (eventTypeId: string): Routing[] =>
  JSON.parse(
    JSON.stringify(routings.filter((r) => r.event_type_id === eventTypeId)),
  );

// ── Templates ──

export const getTemplates = (): Template[] =>
  JSON.parse(JSON.stringify(templates));

export const createTemplate = (data: Omit<Template, "id">): Template => {
  const tpl: Template = { ...data, id: `tpl-${nextTemplateId++}` };
  templates.push(tpl);
  return JSON.parse(JSON.stringify(tpl));
};

export const updateTemplate = (
  id: string,
  data: Partial<Template>,
): void => {
  const idx = templates.findIndex((t) => t.id === id);
  if (idx === -1) throw new Error(`Template ${id} not found`);
  templates[idx] = { ...templates[idx], ...data };
};

export const deleteTemplate = (id: string): void => {
  templates = templates.filter((t) => t.id !== id);
};

// ── History ──

export const getHistory = (
  limit?: number,
  offset?: number,
): NotificationHistory[] => {
  const l = limit ?? 50;
  const o = offset ?? 0;
  return JSON.parse(JSON.stringify(history.slice(o, o + l)));
};

export const getHistoryByEventType = (
  eventTypeId: string,
): NotificationHistory[] =>
  JSON.parse(
    JSON.stringify(history.filter((h) => h.event_type_id === eventTypeId)),
  );

export const clearHistory = (): void => {
  history = [];
};

// ── Settings ──

export const getSettings = (): Settings =>
  JSON.parse(JSON.stringify(settings));

export const getSetting = (key: string): string | null =>
  settings[key] ?? null;

export const setSetting = (key: string, value: string): void => {
  settings[key] = value;
};

export const deleteSetting = (key: string): void => {
  delete settings[key];
};

// ── Hooks ──

export const getHooksStatus = (): HooksStatus =>
  JSON.parse(JSON.stringify(hooksStatus));

export const installHook = (tool: string): void => {
  if (tool in hooksStatus) {
    (hooksStatus as Record<string, boolean>)[tool] = true;
  }
};

export const uninstallHook = (tool: string): void => {
  if (tool in hooksStatus) {
    (hooksStatus as Record<string, boolean>)[tool] = false;
  }
};

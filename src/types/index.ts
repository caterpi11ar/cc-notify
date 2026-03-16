export interface Channel {
  id: string;
  name: string;
  channel_type: string;
  config: Record<string, unknown>;
  enabled: boolean;
  sort_index: number;
  created_at: number;
  updated_at: number;
}

export interface EventType {
  id: string;
  name: string;
  category: string;
  is_builtin: boolean;
  config: Record<string, unknown>;
  enabled: boolean;
}

export interface Rule {
  id: string;
  name: string;
  rule_type: string;
  pattern: string;
  event_type_id: string;
  enabled: boolean;
  created_at: number;
}

export interface Routing {
  event_type_id: string;
  channel_id: string;
  enabled: boolean;
  priority: number;
}

export interface Template {
  id: string;
  name: string;
  channel_type: string;
  body_template: string;
  format: string;
  is_default: boolean;
}

export interface NotificationHistory {
  id: number;
  event_type_id: string;
  channel_id: string;
  status: string;
  message_body: string;
  error_message: string | null;
  metadata: Record<string, unknown>;
  created_at: number;
}

export interface Settings {
  [key: string]: string;
}

export interface HooksStatus {
  claude: boolean;
  codex: boolean;
  gemini: boolean;
}

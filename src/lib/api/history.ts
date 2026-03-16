import { invoke } from "@tauri-apps/api/core";
import type { NotificationHistory } from "@/types";

export const historyApi = {
  async getAll(limit?: number, offset?: number): Promise<NotificationHistory[]> {
    return await invoke("get_history", { limit, offset });
  },
  async getByEventType(eventTypeId: string): Promise<NotificationHistory[]> {
    return await invoke("get_history_by_event_type", { eventTypeId });
  },
  async clear(): Promise<void> {
    return await invoke("clear_history");
  },
};

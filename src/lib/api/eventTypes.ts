import { invoke } from "@tauri-apps/api/core";
import type { EventType } from "@/types";

export const eventTypesApi = {
  async getAll(): Promise<EventType[]> {
    return await invoke("get_event_types");
  },
  async create(
    eventType: Omit<EventType, "id">,
  ): Promise<EventType> {
    return await invoke("create_event_type", { eventType });
  },
  async update(id: string, eventType: Partial<EventType>): Promise<void> {
    return await invoke("update_event_type", { id, eventType });
  },
  async delete(id: string): Promise<void> {
    return await invoke("delete_event_type", { id });
  },
};

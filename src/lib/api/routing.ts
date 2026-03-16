import { invoke } from "@tauri-apps/api/core";
import type { Routing } from "@/types";

export const routingApi = {
  async getAll(): Promise<Routing[]> {
    return await invoke("get_routings");
  },
  async set(routing: Routing): Promise<void> {
    return await invoke("set_routing", { routing });
  },
  async delete(eventTypeId: string, channelId: string): Promise<void> {
    return await invoke("delete_routing", { eventTypeId, channelId });
  },
  async getByEventType(eventTypeId: string): Promise<Routing[]> {
    return await invoke("get_routings_by_event_type", { eventTypeId });
  },
};

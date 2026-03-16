import { invoke } from "@tauri-apps/api/core";
import type { Channel } from "@/types";

export const channelsApi = {
  async getAll(): Promise<Channel[]> {
    return await invoke("get_channels");
  },
  async create(
    channel: Omit<Channel, "id" | "created_at" | "updated_at">,
  ): Promise<Channel> {
    return await invoke("create_channel", { channel });
  },
  async update(id: string, channel: Partial<Channel>): Promise<void> {
    return await invoke("update_channel", { id, channel });
  },
  async delete(id: string): Promise<void> {
    return await invoke("delete_channel", { id });
  },
  async test(
    id: string,
  ): Promise<{ success: boolean; message?: string }> {
    return await invoke("test_channel", { id });
  },
};

import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "@/types";

export const settingsApi = {
  async getAll(): Promise<Settings> {
    return await invoke("get_settings");
  },
  async get(key: string): Promise<string | null> {
    return await invoke("get_setting", { key });
  },
  async set(key: string, value: string): Promise<void> {
    return await invoke("set_setting", { key, value });
  },
  async delete(key: string): Promise<void> {
    return await invoke("delete_setting", { key });
  },
};

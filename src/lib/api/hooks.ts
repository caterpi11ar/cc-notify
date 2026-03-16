import { invoke } from "@tauri-apps/api/core";
import type { HooksStatus } from "@/types";

export const hooksApi = {
  async getStatus(): Promise<HooksStatus> {
    return await invoke("get_hooks_status");
  },
  async install(tool: string): Promise<void> {
    return await invoke("install_hook", { tool });
  },
  async uninstall(tool: string): Promise<void> {
    return await invoke("uninstall_hook", { tool });
  },
};

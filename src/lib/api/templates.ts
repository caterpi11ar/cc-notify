import { invoke } from "@tauri-apps/api/core";
import type { Template } from "@/types";

export const templatesApi = {
  async getAll(): Promise<Template[]> {
    return await invoke("get_templates");
  },
  async create(
    template: Omit<Template, "id">,
  ): Promise<Template> {
    return await invoke("create_template", { template });
  },
  async update(id: string, template: Partial<Template>): Promise<void> {
    return await invoke("update_template", { id, template });
  },
  async delete(id: string): Promise<void> {
    return await invoke("delete_template", { id });
  },
};

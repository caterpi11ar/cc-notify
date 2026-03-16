import { invoke } from "@tauri-apps/api/core";
import type { Rule } from "@/types";

export const rulesApi = {
  async getAll(): Promise<Rule[]> {
    return await invoke("get_rules");
  },
  async create(
    rule: Omit<Rule, "id" | "created_at">,
  ): Promise<Rule> {
    return await invoke("create_rule", { rule });
  },
  async update(id: string, rule: Partial<Rule>): Promise<void> {
    return await invoke("update_rule", { id, rule });
  },
  async delete(id: string): Promise<void> {
    return await invoke("delete_rule", { id });
  },
};

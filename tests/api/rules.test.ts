import { describe, it, expect } from "vitest";
import { rulesApi } from "@/lib/api/rules";

describe("rulesApi", () => {
  it("getAll returns all rules", async () => {
    const rules = await rulesApi.getAll();
    expect(rules).toHaveLength(2);
    expect(rules[0].name).toBe("Error Keyword");
    expect(rules[0].rule_type).toBe("keyword");
  });

  it("create adds a new rule", async () => {
    const created = await rulesApi.create({
      name: "Regex Rule",
      rule_type: "regex",
      pattern: "^fatal.*",
      event_type_id: "evt-2",
      enabled: true,
    });
    expect(created.id).toBeTruthy();
    expect(created.name).toBe("Regex Rule");

    const all = await rulesApi.getAll();
    expect(all).toHaveLength(3);
  });

  it("update modifies a rule", async () => {
    await rulesApi.update("rule-1", { enabled: false });
    const all = await rulesApi.getAll();
    const updated = all.find((r) => r.id === "rule-1");
    expect(updated?.enabled).toBe(false);
  });

  it("delete removes a rule", async () => {
    await rulesApi.delete("rule-2");
    const all = await rulesApi.getAll();
    expect(all).toHaveLength(1);
    expect(all.find((r) => r.id === "rule-2")).toBeUndefined();
  });
});

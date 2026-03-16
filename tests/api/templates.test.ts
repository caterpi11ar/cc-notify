import { describe, it, expect } from "vitest";
import { templatesApi } from "@/lib/api/templates";

describe("templatesApi", () => {
  it("getAll returns all templates", async () => {
    const templates = await templatesApi.getAll();
    expect(templates).toHaveLength(1);
    expect(templates[0].name).toBe("Default Slack");
    expect(templates[0].is_default).toBe(true);
  });

  it("create adds a new template", async () => {
    const created = await templatesApi.create({
      name: "Discord Template",
      channel_type: "discord",
      body_template: "**{{event_name}}**: {{message}}",
      format: "markdown",
      is_default: false,
    });
    expect(created.id).toBeTruthy();
    expect(created.name).toBe("Discord Template");

    const all = await templatesApi.getAll();
    expect(all).toHaveLength(2);
  });

  it("update modifies a template", async () => {
    await templatesApi.update("tpl-1", {
      body_template: "Updated: {{message}}",
    });
    const all = await templatesApi.getAll();
    const updated = all.find((t) => t.id === "tpl-1");
    expect(updated?.body_template).toBe("Updated: {{message}}");
  });

  it("delete removes a template", async () => {
    await templatesApi.delete("tpl-1");
    const all = await templatesApi.getAll();
    expect(all).toHaveLength(0);
  });
});

import { describe, it, expect } from "vitest";
import { eventTypesApi } from "@/lib/api/eventTypes";

describe("eventTypesApi", () => {
  it("getAll returns all event types", async () => {
    const events = await eventTypesApi.getAll();
    expect(events).toHaveLength(4);
    expect(events[0].name).toBe("Task Complete");
    expect(events[0].category).toBe("claude_hook");
  });

  it("create adds a new event type", async () => {
    const created = await eventTypesApi.create({
      name: "New Event",
      category: "custom",
      is_builtin: false,
      config: {},
      enabled: true,
    });
    expect(created.id).toBeTruthy();
    expect(created.name).toBe("New Event");

    const all = await eventTypesApi.getAll();
    expect(all).toHaveLength(5);
  });

  it("update modifies an event type", async () => {
    await eventTypesApi.update("evt-4", { enabled: true });
    const all = await eventTypesApi.getAll();
    const updated = all.find((e) => e.id === "evt-4");
    expect(updated?.enabled).toBe(true);
  });

  it("delete removes an event type", async () => {
    await eventTypesApi.delete("evt-4");
    const all = await eventTypesApi.getAll();
    expect(all).toHaveLength(3);
    expect(all.find((e) => e.id === "evt-4")).toBeUndefined();
  });
});

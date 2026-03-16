import { describe, it, expect } from "vitest";
import { routingApi } from "@/lib/api/routing";

describe("routingApi", () => {
  it("getAll returns all routings", async () => {
    const routings = await routingApi.getAll();
    expect(routings).toHaveLength(3);
    expect(routings[0].event_type_id).toBe("evt-1");
    expect(routings[0].channel_id).toBe("ch-1");
  });

  it("set adds or updates a routing", async () => {
    await routingApi.set({
      event_type_id: "evt-3",
      channel_id: "ch-1",
      enabled: true,
      priority: 0,
    });
    const all = await routingApi.getAll();
    expect(all).toHaveLength(4);
    expect(all.find((r) => r.event_type_id === "evt-3")).toBeTruthy();
  });

  it("delete removes a routing", async () => {
    await routingApi.delete("evt-1", "ch-1");
    const all = await routingApi.getAll();
    expect(all).toHaveLength(2);
    expect(
      all.find(
        (r) => r.event_type_id === "evt-1" && r.channel_id === "ch-1",
      ),
    ).toBeUndefined();
  });

  it("getByEventType returns routings for a specific event type", async () => {
    const routings = await routingApi.getByEventType("evt-1");
    expect(routings).toHaveLength(2);
    expect(routings.every((r) => r.event_type_id === "evt-1")).toBe(true);
  });
});

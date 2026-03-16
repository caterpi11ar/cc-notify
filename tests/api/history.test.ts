import { describe, it, expect } from "vitest";
import { historyApi } from "@/lib/api/history";

describe("historyApi", () => {
  it("getAll returns history with default limit", async () => {
    const history = await historyApi.getAll();
    expect(history).toHaveLength(3);
    expect(history[0].status).toBe("sent");
    expect(history[1].status).toBe("failed");
  });

  it("getAll respects limit and offset", async () => {
    const history = await historyApi.getAll(2, 0);
    expect(history).toHaveLength(2);

    const offset = await historyApi.getAll(2, 2);
    expect(offset).toHaveLength(1);
    expect(offset[0].id).toBe(3);
  });

  it("getByEventType filters history by event type", async () => {
    const history = await historyApi.getByEventType("evt-1");
    expect(history).toHaveLength(2);
    expect(history.every((h) => h.event_type_id === "evt-1")).toBe(true);
  });

  it("clear removes all history", async () => {
    await historyApi.clear();
    const history = await historyApi.getAll();
    expect(history).toHaveLength(0);
  });
});

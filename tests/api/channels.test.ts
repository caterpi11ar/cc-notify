import { describe, it, expect } from "vitest";
import { channelsApi } from "@/lib/api/channels";

describe("channelsApi", () => {
  it("getAll returns all channels", async () => {
    const channels = await channelsApi.getAll();
    expect(channels).toHaveLength(3);
    expect(channels[0].name).toBe("Native Notification");
    expect(channels[1].channel_type).toBe("slack");
    expect(channels[2].enabled).toBe(false);
  });

  it("create adds a new channel and returns it", async () => {
    const created = await channelsApi.create({
      name: "Test Webhook",
      channel_type: "webhook",
      config: { url: "https://example.com" },
      enabled: true,
      sort_index: 3,
    });
    expect(created.id).toBeTruthy();
    expect(created.name).toBe("Test Webhook");
    expect(created.channel_type).toBe("webhook");

    const all = await channelsApi.getAll();
    expect(all).toHaveLength(4);
  });

  it("update modifies a channel", async () => {
    await channelsApi.update("ch-1", { name: "Updated Native" });
    const all = await channelsApi.getAll();
    const updated = all.find((c) => c.id === "ch-1");
    expect(updated?.name).toBe("Updated Native");
  });

  it("delete removes a channel", async () => {
    await channelsApi.delete("ch-3");
    const all = await channelsApi.getAll();
    expect(all).toHaveLength(2);
    expect(all.find((c) => c.id === "ch-3")).toBeUndefined();
  });

  it("test returns success result", async () => {
    const result = await channelsApi.test("ch-1");
    expect(result.success).toBe(true);
    expect(result.message).toBeTruthy();
  });
});

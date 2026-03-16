import { describe, it, expect } from "vitest";
import { settingsApi } from "@/lib/api/settings";

describe("settingsApi", () => {
  it("getAll returns all settings", async () => {
    const settings = await settingsApi.getAll();
    expect(settings.language).toBe("en");
    expect(settings.kill_switch).toBe("false");
    expect(settings.sound_enabled).toBe("true");
  });

  it("get returns a single setting value", async () => {
    const value = await settingsApi.get("language");
    expect(value).toBe("en");

    const missing = await settingsApi.get("nonexistent");
    expect(missing).toBeNull();
  });

  it("set creates or updates a setting", async () => {
    await settingsApi.set("language", "zh");
    const value = await settingsApi.get("language");
    expect(value).toBe("zh");

    await settingsApi.set("new_key", "new_value");
    const newValue = await settingsApi.get("new_key");
    expect(newValue).toBe("new_value");
  });

  it("delete removes a setting", async () => {
    await settingsApi.delete("voice_name");
    const value = await settingsApi.get("voice_name");
    expect(value).toBeNull();
  });
});

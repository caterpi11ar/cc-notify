import { describe, it, expect } from "vitest";
import { hooksApi } from "@/lib/api/hooks";

describe("hooksApi", () => {
  it("getStatus returns hooks installation status", async () => {
    const status = await hooksApi.getStatus();
    expect(status.claude).toBe(true);
    expect(status.codex).toBe(false);
    expect(status.gemini).toBe(false);
  });

  it("install sets a hook to installed", async () => {
    await hooksApi.install("codex");
    const status = await hooksApi.getStatus();
    expect(status.codex).toBe(true);
  });

  it("uninstall sets a hook to not installed", async () => {
    await hooksApi.uninstall("claude");
    const status = await hooksApi.getStatus();
    expect(status.claude).toBe(false);
  });
});

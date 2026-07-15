import { describe, expect, it } from "vitest";
import { gatewayApi } from "@/lib/api/gateway";

describe("gatewayApi", () => {
  it("getStatus returns local base URLs", async () => {
    const status = await gatewayApi.getStatus();

    expect(status.running).toBe(false);
    expect(status.openai_base_url).toBe("http://127.0.0.1:17777/v1");
    expect(status.anthropic_base_url).toBe("http://127.0.0.1:17777");
  });

  it("start returns running status", async () => {
    const status = await gatewayApi.start();

    expect(status.running).toBe(true);
    expect(status.port).toBe(17777);
  });

  it("returns and regenerates local API key state", async () => {
    const status = await gatewayApi.getApiKeyStatus();
    const key = await gatewayApi.regenerateApiKey();

    expect(status.exists).toBe(true);
    expect(status.preview).toBe("lgw_test...1234");
    expect(key).toBe("lgw_new_key");
  });

  it("lists the configured upstream provider", async () => {
    const providers = await gatewayApi.getProviders();
    expect(providers[0].name).toBe("Local OpenAI");
    expect(providers[0].protocol).toBe("openai_compatible");
  });

  it("returns usage logs", async () => {
    const logs = await gatewayApi.getLogs();

    expect(logs[0].endpoint).toBe("/v1/chat/completions");
    expect(logs[0].resolved_provider).toBe("Local OpenAI");
    expect(logs[0].input_tokens).toBe(4);
    expect(logs[0].output_tokens).toBe(2);
    expect(logs[0].total_tokens).toBe(6);
  });
});

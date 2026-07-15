import { generateText } from "ai";
import { createOpenAI } from "@ai-sdk/openai";

export const runtime = "nodejs";

const defaultPrompt = "Explain the concept of quantum entanglement.";

export async function POST(request: Request) {
  const body = (await request.json().catch(() => ({}))) as {
    apiKey?: string;
    baseURL?: string;
    prompt?: string;
    model?: string;
  };

  const baseURL =
    body.baseURL?.trim() ||
    process.env.LOCAL_GATEWAY_BASE_URL ||
    "http://127.0.0.1:17777/v1";
  const apiKey = body.apiKey?.trim() || process.env.LOCAL_GATEWAY_API_KEY;
  const modelName = body.model?.trim();
  const prompt = body.prompt?.trim() || defaultPrompt;

  if (!modelName) {
    return Response.json(
      {
        error: "Model is required. Enter a model name in the test page.",
      },
      { status: 400 },
    );
  }

  if (!apiKey) {
    return Response.json(
      {
        error:
          "LOCAL_GATEWAY_API_KEY is required. Copy the local gateway key from the desktop app.",
      },
      { status: 400 },
    );
  }

  const gateway = createOpenAI({
    baseURL,
    apiKey,
  });

  const startedAt = Date.now();
  const { text, usage, finishReason } = await generateText({
    model: gateway(modelName),
    prompt,
  });

  return Response.json({
    text,
    usage,
    finishReason,
    model: modelName,
    baseURL,
    latencyMs: Date.now() - startedAt,
  });
}

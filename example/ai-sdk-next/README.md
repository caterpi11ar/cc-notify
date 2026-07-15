# Local AI Gateway AI SDK Example

This is a small Next.js app for testing the local gateway with Vercel AI SDK.

The server route uses:

```ts
import { generateText } from "ai";

const { text } = await generateText({
  model: gateway(modelName),
  prompt: "Explain the concept of quantum entanglement.",
});
```

`gateway(...)` is an OpenAI-compatible provider configured to call the local gateway at `http://127.0.0.1:17777/v1`.
`modelName` comes from the model input on the page, so you can test any model routed by your gateway, for example `gpt-5.5`.

## Run

```bash
cd example/ai-sdk-next
pnpm install
cp .env.example .env.local
pnpm dev
```

Paste the local gateway key into the page. You can also set `LOCAL_GATEWAY_API_KEY` in `.env.local` if you do not want to type it each time.

Then open:

```text
http://localhost:4000
```

Enter the local API key, gateway base URL, and model name on the page. For model, use any model routed by your gateway, for example:

```text
gpt-5.5
```

The default prompt is:

```text
Explain the concept of quantum entanglement.
```

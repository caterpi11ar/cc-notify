# local-ai-gateway

Transparent local AI gateway for desktop workflows.

`local-ai-gateway` runs a localhost service that accepts OpenAI-compatible requests, validates a local API key, forwards requests to the configured upstream, and records usage logs. Upstream selection/configuration is intentionally kept outside the UI so tools such as `cc switch` can own provider setup.

## What It Does

- Starts a local gateway on `127.0.0.1`, default port `17777`
- Exposes OpenAI-compatible base URL: `http://127.0.0.1:17777/v1`
- Accepts local auth via `Authorization: Bearer <local_api_key>` or `x-api-key`
- Transparently forwards `/v1/*` requests to the configured upstream
- Records usage, latency, status, model, provider, and token metrics
- Provides a desktop dashboard for start/stop/restart, local key copy, and usage inspection

## What It Does Not Do

- It does not manage provider configuration in the desktop UI
- It does not implement model aliasing, routing rules, or protocol conversion in the UI
- It does not replace `cc switch`; upstream configuration should be handled by that layer

## Development

```bash
pnpm install
pnpm dev
```

The desktop renderer runs on `http://localhost:1111` during development.

## AI SDK Example

A Next.js test app is available in `example/ai-sdk-next`.

```bash
cd example/ai-sdk-next
pnpm install
pnpm dev
```

Open:

```text
http://localhost:4000
```

The page lets you enter:

- Local gateway API key
- Gateway base URL
- Model name
- Prompt

It calls Vercel AI SDK `generateText` through the local gateway.

## Tests

```bash
pnpm typecheck
pnpm vitest run tests/api
cargo test --manifest-path src-tauri/Cargo.toml
```

Real upstream tests are available in Rust and are ignored unless the required environment variables are provided.

## Project Layout

```text
src/                         Desktop React UI
src-tauri/                   Tauri/Rust gateway backend
example/ai-sdk-next/         Next.js + AI SDK gateway test page
tests/                       Frontend API logic tests
```

## Local Data

Runtime data is stored under:

```text
~/.local-ai-gateway/
```

The SQLite database is created by the desktop app and includes gateway state, upstream metadata, and usage logs.

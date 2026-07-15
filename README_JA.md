# local-ai-gateway

デスクトップ向けの透過的なローカル AI ゲートウェイです。

`local-ai-gateway` は localhost サービスを起動し、ローカル API Key を検証して、OpenAI-compatible リクエストを設定済みの upstream へ転送し、usage ログを記録します。upstream の設定はデスクトップ UI では管理せず、`cc switch` のような外部レイヤーに任せます。

## 機能

- 既定で `127.0.0.1:17777` を listen
- OpenAI-compatible Base URL: `http://127.0.0.1:17777/v1`
- `Authorization: Bearer <local_api_key>` と `x-api-key` に対応
- `/v1/*` リクエストを upstream に透過転送
- usage、latency、status、model、provider、tokens を記録
- デスクトップ UI で start/stop/restart、local key コピー、usage 確認

## 対象外

- デスクトップ UI で Provider 設定は管理しない
- model alias、routing rule、protocol conversion の設定は持たない
- `cc switch` を置き換えず、ローカル gateway と observability に集中する

## 開発

```bash
pnpm install
pnpm dev
```

開発時の renderer:

```text
http://localhost:1111
```

## AI SDK Example

サンプルは `example/ai-sdk-next` にあります。

```bash
cd example/ai-sdk-next
pnpm install
pnpm dev
```

Open:

```text
http://localhost:4000
```

ページで local API key、gateway base URL、model、prompt を入力し、Vercel AI SDK `generateText` でローカル gateway を呼び出せます。

## Tests

```bash
pnpm typecheck
pnpm vitest run tests/api
cargo test --manifest-path src-tauri/Cargo.toml
```

## Layout

```text
src/                         Desktop React UI
src-tauri/                   Tauri/Rust gateway backend
local-ai-gateway-cli/        Hook workflows CLI
example/ai-sdk-next/         Next.js + AI SDK test page
tests/                       Frontend API logic tests
```

## Local Data

Runtime data is stored under:

```text
~/.local-ai-gateway/
```

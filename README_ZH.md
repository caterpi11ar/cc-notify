# local-ai-gateway

桌面端透明本地 AI 网关。

`local-ai-gateway` 在本机启动 localhost 服务，校验本地 API Key，将 OpenAI-compatible 请求转发到当前配置的上游，并记录 usage 日志。上游配置不放在桌面 UI 里维护，应由 `cc switch` 这类产品负责。

## 功能

- 默认监听 `127.0.0.1:17777`
- OpenAI-compatible Base URL：`http://127.0.0.1:17777/v1`
- 支持 `Authorization: Bearer <local_api_key>` 和 `x-api-key`
- 透明转发 `/v1/*` 请求到上游
- 记录 usage、latency、status、model、provider、tokens
- 桌面 UI 提供启动/停止/重启、本地 Key 复制、Usage 查看

## 非目标

- 不在桌面 UI 中配置 Provider
- 不做模型别名、路由规则、协议转换配置
- 不替代 `cc switch`，只负责本地网关和可观测性

## 开发

```bash
pnpm install
pnpm dev
```

开发时桌面前端运行在：

```text
http://localhost:1111
```

## AI SDK 示例

示例项目位于 `example/ai-sdk-next`。

```bash
cd example/ai-sdk-next
pnpm install
pnpm dev
```

打开：

```text
http://localhost:4000
```

页面可以输入本地 API Key、Gateway Base URL、模型名和 Prompt，并通过 Vercel AI SDK `generateText` 调用本地网关。

## 测试

```bash
pnpm typecheck
pnpm vitest run tests/api
cargo test --manifest-path src-tauri/Cargo.toml
```

## 目录

```text
src/                         桌面 React UI
src-tauri/                   Tauri/Rust 网关后端
example/ai-sdk-next/         Next.js + AI SDK 测试页面
tests/                       前端 API 逻辑测试
```

## 本地数据

运行数据默认存储在：

```text
~/.local-ai-gateway/
```

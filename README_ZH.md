<div align="center">

# CC Notify

### AI CLI 工具通知管理器

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/caterpi11ar/cc-notify/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/caterpi11ar/cc-notify/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-orange.svg)](https://tauri.app/)
[![Tests](https://img.shields.io/badge/tests-163%20passing-brightgreen.svg)]()

[English](README.md) | 中文 | [日本語](README_JA.md)

</div>

## 为什么需要 CC Notify？

Claude Code、Codex、Gemini CLI 等 AI CLI 工具经常在后台执行长时间任务——但除非你一直盯着终端，否则无法知道它们何时完成、遇到错误或需要你的输入。

**CC Notify** 通过挂钩这些 CLI 工具，将通知路由到你实际使用的渠道来解决这个问题：系统通知、Slack、Discord、Telegram、飞书、Webhook、声音提醒、语音播报和 Dock 角标计数。

- **不错过任何任务完成** — AI 代理完成、出错或需要权限时立即通知
- **多渠道路由** — 将不同事件路由到不同渠道（如：错误 → Slack，完成 → 系统通知）
- **自定义规则** — 关键词、正则、文件变更触发器，实现精细化通知控制
- **Webhook 模板** — 内置飞书支持，可扩展的模板系统便于未来集成更多渠道
- **独立 CLI** — 无需桌面应用，通过 `cc-notify` CLI 即可工作，适合 SSH 和无头服务器
- **跨平台** — 基于 Tauri 2 的原生桌面应用，支持 Windows、macOS 和 Linux

## 功能特性

### 通知渠道

| 渠道 | 说明 |
|------|------|
| **系统原生** | 通过 Tauri 通知插件的系统级通知 |
| **Slack** | 基于 Webhook 的消息，支持频道和 @ 提醒 |
| **Discord** | 富嵌入通知，支持自定义用户名和颜色 |
| **Microsoft Teams** | Incoming Webhook 连接器 |
| **Telegram** | Bot API，可配置解析模式（HTML/Markdown） |
| **HTTP Webhook** | 通用 Webhook，支持自定义方法、请求头和模板 |
| **Webhook: 飞书** | 飞书机器人 Webhook，支持交互式卡片格式 |
| **声音** | 跨平台声音提醒（macOS/Linux/Windows） |
| **语音** | 语音播报（支持系统 TTS 与本地语音包，macOS） |
| **托盘徽标** | macOS Dock 角标计数，随通知递增 |

### 自定义语音包（macOS）

在 **Channels** 的 `voice` 通道中可配置：

- `mode`：`tts` 或 `voice_pack`
- `voice_pack_dir`：本地语音包目录（`voice_pack` 模式必填）
- `voice` / `rate`：`tts` 模式下使用

语音包按事件 ID 命名：

```text
<voice_pack_dir>/
  stop.wav
  notification.idle_prompt.aiff
  notification.permission_prompt.m4a
  default.mp3
```

单个事件的查找顺序：
1. `<event_id>.wav` -> `.aiff` -> `.m4a` -> `.mp3`
2. `default.wav` -> `.aiff` -> `.m4a` -> `.mp3`
3. 仍未命中则静默跳过（不播放）

### 事件类型与路由

- **12 种内置事件类型**，分为 3 个类别：
  - **Claude Code Hooks**：任务完成、空闲等待、权限请求、认证成功、MCP 输入、子代理停止、会话开始/结束
  - **扩展事件**：长时间运行、错误、Token 阈值、费用阈值
- **事件 → 渠道路由**，支持优先级
- **按事件启用/禁用**

### 自定义规则

- **关键词匹配** — 输出中包含特定词时触发
- **正则匹配** — 基于模式的触发器
- **文件变更** — 基于 Glob 的文件系统触发器
- **自定义事件** — 用户自定义事件类型

### 独立 CLI

`cc-notify` CLI 可独立工作——无需桌面应用：

```bash
# 发送测试通知
cc-notify test

# 通过 Claude Code hooks 自动触发
# （在 设置 → Hooks 集成 中安装）
```

### 国际化

完整支持**英文**、**简体中文**和**日语**。

## 快速开始

### 环境要求

- **Node.js** 18+
- **pnpm** 8+
- **Rust** 1.85+
- **Tauri CLI** 2.8+

### 安装与运行

```bash
# 克隆仓库
git clone https://github.com/caterpi11ar/cc-notify.git
cd cc-notify

# 安装依赖
pnpm install

# 开发模式运行
pnpm dev
```

### 基本用法

1. **通道** — 添加通知渠道（Slack、Discord、Telegram 等）并切换启用/禁用
2. **事件** — 配置哪些事件路由到哪些渠道
3. **规则** — 设置自定义触发器（关键词、正则、文件变更）
4. **Hooks** — 在设置页面为 Claude Code / Codex / Gemini CLI 安装 hooks
5. **历史** — 查看通知发送历史和状态

## 数据存储

| 路径 | 内容 |
|------|------|
| `~/.cc-notify/cc-notify.db` | SQLite 数据库（渠道、事件、规则、路由、历史、设置） |
| `~/.cc-notify/disabled` | 总开关文件（创建即禁用所有通知） |
| `~/.cc-notify/last_notification` | 频率限制时间戳 |

## 常见问题

<details>
<summary><strong>支持哪些 AI CLI 工具？</strong></summary>

CC Notify 支持 **Claude Code**、**Codex** 和 **Gemini CLI**。可在 设置 → Hooks 集成 中安装钩子。

</details>

<details>
<summary><strong>CLI 可以不依赖桌面应用使用吗？</strong></summary>

可以。`cc-notify` CLI 读取相同的 SQLite 数据库并独立发送通知。如果数据库不存在，会回退到系统原生通知。

</details>

<details>
<summary><strong>飞书集成是怎么工作的？</strong></summary>

飞书实现为 **Webhook 模板** — 创建 Webhook 渠道时选择"飞书"模板。后端会使用飞书机器人 API 格式（CLI 中为交互式卡片，Tauri 中为文本消息），并验证 Webhook URL 是否为飞书/Larksuite 域名。

</details>

<details>
<summary><strong>macOS 提示"无法打开"或"未知开发者"</strong></summary>

应用尚未使用 Apple 开发者证书签名，macOS Gatekeeper 会阻止从网上下载的未签名应用。

**方法一** — 右键点击应用 → 选择"打开" → 在弹窗中点击"打开"（只需一次）。

**方法二** — 前往 **系统设置 → 隐私与安全性**，下滑找到 **"仍要打开"** 按钮。

**方法三** — 在终端中移除隔离标记：

```bash
xattr -cr "/Applications/CC Notify.app"
```

以上任一方法操作后，后续即可正常打开。

</details>

## 贡献

欢迎提交 Issue 和 Pull Request！

提交 PR 前请确保：

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path cc-notify-cli/Cargo.toml
pnpm test:unit
```

## 许可证

MIT

<div align="center">

# CC Notify

### AI CLIツール通知マネージャー

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/caterpi11ar/cc-notify/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/caterpi11ar/cc-notify/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-orange.svg)](https://tauri.app/)
[![Tests](https://img.shields.io/badge/tests-163%20passing-brightgreen.svg)]()

[English](README.md) | [中文](README_ZH.md) | 日本語

</div>

## CC Notifyとは？

Claude Code、Codex、Gemini CLIなどのAI CLIツールはバックグラウンドで長時間タスクを実行しますが、ターミナルを見続けない限り、完了・エラー・入力待ちを知ることができません。

**CC Notify**はこれらのCLIツールにフックし、実際に使用しているチャンネルに通知をルーティングすることで、この問題を解決します：システム通知、Slack、Discord、Telegram、Feishu、Webhook、サウンドアラート、音声読み上げ、Dockバッジカウント。

- **タスク完了を見逃さない** — AIエージェントの完了、エラー、権限要求を即座に通知
- **マルチチャンネルルーティング** — 異なるイベントを異なるチャンネルに（例：エラー→Slack、完了→システム通知）
- **カスタムルール** — キーワード、正規表現、ファイル変更トリガーによる細かな通知制御
- **Webhookテンプレート** — Feishu（Lark）の組み込みサポート、拡張可能なテンプレートシステム
- **スタンドアロンCLI** — デスクトップアプリ不要、`cc-notify` CLIで動作、SSH・ヘッドレスサーバーに最適
- **クロスプラットフォーム** — Tauri 2で構築されたネイティブデスクトップアプリ（Windows、macOS、Linux）

## 機能

### 通知チャンネル

| チャンネル | 説明 |
|-----------|------|
| **システム通知** | Tauri通知プラグインによるOS通知 |
| **Slack** | Webhookメッセージ、チャンネル・メンション対応 |
| **Discord** | リッチ埋め込み通知、カスタムユーザー名・カラー |
| **Microsoft Teams** | Incoming Webhookコネクタ |
| **Telegram** | Bot API、パースモード設定可能（HTML/Markdown） |
| **HTTP Webhook** | 汎用Webhook、カスタムメソッド・ヘッダー・テンプレート |
| **Webhook: Feishu** | Feishu（ラーク）ボットWebhook、インタラクティブカード形式 |
| **サウンド** | クロスプラットフォームサウンドアラート |
| **音声** | テキスト読み上げ（macOS `say`コマンド） |
| **トレイバッジ** | macOS Dockバッジカウント、通知ごとに増加 |

### イベントタイプとルーティング

- **12種類の組み込みイベント**（3カテゴリ）：
  - **Claude Code Hooks**：タスク完了、アイドル待ち、権限要求、認証成功、MCP入力、サブエージェント停止、セッション開始/終了
  - **拡張イベント**：長時間実行、エラー、トークン閾値、コスト閾値
- **イベント→チャンネルルーティング**（優先度付き）
- **イベント単位の有効/無効切り替え**

### カスタムルール

- **キーワード一致** — 出力内の特定ワードでトリガー
- **正規表現一致** — パターンベースのトリガー
- **ファイル変更** — Globベースのファイルシステムトリガー
- **カスタムイベント** — ユーザー定義イベントタイプ

### スタンドアロンCLI

`cc-notify` CLIバイナリは独立動作します：

```bash
# テスト通知を送信
cc-notify test

# Claude Code hooksで自動トリガー
# （設定→Hooks統合でインストール）
```

### 国際化

**英語**、**中国語（簡体字）**、**日本語**を完全サポート。

## クイックスタート

### 必要環境

- **Node.js** 18+
- **pnpm** 8+
- **Rust** 1.85+
- **Tauri CLI** 2.8+

### インストールと実行

```bash
# リポジトリをクローン
git clone https://github.com/caterpi11ar/cc-notify.git
cd cc-notify

# 依存関係をインストール
pnpm install

# 開発モードで実行
pnpm dev
```

### 基本的な使い方

1. **チャンネル** — 通知チャンネル（Slack、Discord、Telegramなど）を追加し、有効/無効を切り替え
2. **イベント** — どのイベントをどのチャンネルにルーティングするか設定
3. **ルール** — カスタムトリガーを設定（キーワード、正規表現、ファイル変更）
4. **Hooks** — 設定ページでClaude Code / Codex / Gemini CLIにhooksをインストール
5. **履歴** — 通知送信履歴とステータスを確認

## データ保存先

| パス | 内容 |
|------|------|
| `~/.cc-notify/cc-notify.db` | SQLiteデータベース（チャンネル、イベント、ルール、ルーティング、履歴、設定） |
| `~/.cc-notify/disabled` | キルスイッチファイル（作成すると全通知無効化） |
| `~/.cc-notify/last_notification` | レート制限タイムスタンプ |

## FAQ

<details>
<summary><strong>どのAI CLIツールに対応していますか？</strong></summary>

CC Notifyは**Claude Code**、**Codex**、**Gemini CLI**をサポートしています。設定→Hooks統合からフックをインストールできます。

</details>

<details>
<summary><strong>CLIはデスクトップアプリなしで使えますか？</strong></summary>

はい。`cc-notify` CLIは同じSQLiteデータベースを読み込み、独立して通知を送信します。データベースがない場合はシステム通知にフォールバックします。

</details>

<details>
<summary><strong>macOSで「開発元不明」の警告が出ます</strong></summary>

**システム設定→プライバシーとセキュリティ→このまま開く**をクリックしてください。Apple開発者証明書での署名がまだ完了していないためです。

</details>

## コントリビューション

IssueとPull Requestを歓迎します！

PR提出前にご確認ください：

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path cc-notify-cli/Cargo.toml
pnpm test:unit
```

## ライセンス

MIT

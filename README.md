<div align="center">

# mix_api_bridge

[![EN](https://img.shields.io/badge/EN-English-blue?style=for-the-badge)](#english)
[![ZH](https://img.shields.io/badge/ZH-中文文档-orange?style=for-the-badge)](#chinese)

</div>

---

<h1 id="english">English</h1>

<div align="center">

**Run Xiaomi miclaw's models locally as an OpenAI / Anthropic / OpenCode-compatible endpoint.**

Sign in with a miclaw-permissioned Xiaomi account, then hit `http://127.0.0.1:8765` from any browser, OpenAI client, Claude-compatible client, or **OpenCode**.

[![Rust](https://img.shields.io/badge/Rust-1.77+-orange?logo=rust)](https://www.rust-lang.org/)
[![Vue 3](https://img.shields.io/badge/Vue-3-42b883?logo=vue.js)](https://vuejs.org/)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)]()
[![License](https://img.shields.io/badge/license-MIT-green)](#license)

</div>

---

## What it does

mix_api_bridge logs into your Xiaomi account the same way the official **Xiaomi miclaw** desktop client does, then exposes the resulting `mimo` LLM through familiar local HTTP endpoints:

- `POST /v1/chat/completions` — OpenAI Chat Completions (drop-in for Cline, Cherry Studio, OpenAI SDKs, **OpenCode**, …)
- `POST /v1/responses` — OpenAI Responses API (native passthrough when available, Chat Completions compatibility fallback otherwise)
- `POST /v1/messages` — Anthropic Messages, with full SSE event translation (drop-in for Claude Code and any client honoring `ANTHROPIC_BASE_URL`)
- `GET /v1/models` — the eight verified model ids

> ⚠️ **Account requirement**: your Xiaomi account must already be approved for miclaw access.

### Models

| Model id | Notes |
|---|---|
| `xiaomi/mimo` | Multimodal, 256 K context (default) |
| `xiaomi/mimo-pro` | Reasoning model with `thinking` traces, 256 K context |
| `xiaomi/mimo-claw-0301` | Claw 0301 reasoning snapshot |
| `xiaomi/MiniMax-M2.5` | MiniMax M2.5, 128 K context |
| `xiaomi/kimi-k2.5` | Kimi K2.5 reasoning, 128 K context |
| `xiaomi/glm-5` | GLM-5, 128 K context |
| `mimo-omni` | Alias → `xiaomi/mimo` |
| `mimo-pro` | Alias → `xiaomi/mimo-pro` |

## Features

- 🔐 **Real Xiaomi OAuth** — username + password + SMS / email 2FA
- 🔄 **Auto token refresh** — `serviceToken` rotated transparently on 401
- 🔑 **Keychain-backed storage** — never on disk in plaintext
- 🔌 **Two protocols, one bridge** — OpenAI Chat Completions and Anthropic Messages
- 🔒 **Optional HTTPS** — TLS with auto self-signed or custom PEM
- 🛡 **Admin password** — Argon2-hashed WebUI control plane
- 🪪 **API-key auth** — optional Bearer auth on `/v1`
- 📊 **Per-model token usage** — windowed charts (1h/1d/7d/30d)
- 🌐 **Browser WebUI** — full console at `http://127.0.0.1:8765`
- 📡 **Live request log** — real-time SSE log in WebUI
- 🧩 **Headless deployment** — server binary for non-graphical machines
- 🖥 **Optional desktop tray**
- 🤖 **OpenCode-ready** — seamless OpenAI-compatible integration

## Quick start

```bash
./mix_api_bridge server
# Open http://127.0.0.1:8765, sign in with Xiaomi account
```

### Client config

**OpenAI-compatible (Cline, Cherry Studio, OpenCode, …)**
```
Base URL: http://127.0.0.1:8765/v1
API key:  anything
Model:    mimo-pro
```

**Anthropic-compatible (Claude Code)**
```
ANTHROPIC_BASE_URL=http://127.0.0.1:8765
ANTHROPIC_API_KEY=anything
```

**OpenCode** — add to `~/.opencode.json`:
```json
{
  "providers": {
    "xiaomi": {
      "type": "openai",
      "api_base": "http://127.0.0.1:8765/v1",
      "api_key": "sk-xiaomi-miclaw",
      "models": [
        { "name": "mimo-pro", "max_input_tokens": 256000 },
        { "name": "xiaomi/mimo", "max_input_tokens": 256000 }
      ]
    }
  },
  "default_provider": "xiaomi",
  "default_model": "mimo-pro"
}
```
Then `opencode --provider xiaomi --model mimo-pro`.

## Build from source

Prerequisites: Rust 1.77+, Node.js 20+, pnpm 9+.

```bash
git clone <this-repo> && cd mix_api_bridge
pnpm install && pnpm build
cd src-tauri && cargo build --release --bin mix_api_bridge
```

## FAQ

**Is this a fork of miclaw?** No — independent client, same protocol, no copied code.

**Need the official miclaw app?** No, but your account must have miclaw access.

**Where are credentials stored?** OS keyring (macOS/Win/Linux) or `/data` volume in Docker.

## License

[MIT](LICENSE)

---

<h1 id="chinese">中文</h1>

<div align="center">

**将小米 miclaw（米络）的模型作为 OpenAI / Anthropic / OpenCode 兼容的本地端点运行。**

使用已获得 miclaw 权限的小米账号登录，然后访问 `http://127.0.0.1:8765`。

[![Rust](https://img.shields.io/badge/Rust-1.77+-orange?logo=rust)](https://www.rust-lang.org/)
[![Vue 3](https://img.shields.io/badge/Vue-3-42b883?logo=vue.js)](https://vuejs.org/)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)]()
[![License](https://img.shields.io/badge/license-MIT-green)](#license)

</div>

---

## 它能做什么

mix_api_bridge 通过 OAuth 登录小米账号，将 `mimo` 系列 LLM 以本地 HTTP 端点暴露：

- `POST /v1/chat/completions` — OpenAI Chat Completions（支持 Cline、Cherry Studio、OpenCode 等）
- `POST /v1/responses` — OpenAI Responses API
- `POST /v1/messages` — Anthropic Messages（支持 Claude Code）
- `GET /v1/models` — 八个模型 ID

> ⚠️ **账号要求**：需获得 miclaw 内测权限。

### 模型列表

| Model id | 说明 |
|---|---|
| `xiaomi/mimo` | 多模态，256K 上下文（默认） |
| `xiaomi/mimo-pro` | 推理模型，256K 上下文 |
| `xiaomi/mimo-claw-0301` | Claw 0301 推理快照 |
| `xiaomi/MiniMax-M2.5` | MiniMax M2.5，128K |
| `xiaomi/kimi-k2.5` | Kimi K2.5，128K |
| `xiaomi/glm-5` | GLM-5，128K |
| `mimo-omni` | 别名 → `xiaomi/mimo` |
| `mimo-pro` | 别名 → `xiaomi/mimo-pro` |

## 功能特性

- 🔐 **真实小米 OAuth** — 用户名+密码+短信/邮箱双因素认证
- 🔄 **自动令牌刷新** — 401 时透明自动轮换
- 🔑 **密钥环存储** — 密钥环加密，永不磁盘明文
- 🔌 **双协议桥接** — OpenAI + Anthropic 双兼容
- 🔒 **可选 HTTPS** — 自签名或自定义证书
- 🛡 **管理员密码** — Argon2 加密 + Cookie 会话
- 🪪 **API Key 认证** — 可选 Bearer 认证
- 📊 **逐模型 Token 统计** — 仪表盘图表
- 🌐 **浏览器 WebUI** — `http://127.0.0.1:8765`
- 📡 **实时请求日志** — SSE 实时流
- 🧩 **无头部署** — 无需图形界面
- 🖥 **可选桌面托盘**
- 🤖 **OpenCode 原生支持**

## 快速开始

```bash
./mix_api_bridge server
# 浏览器打开 http://127.0.0.1:8765，登录小米账号
```

### 客户端配置

**OpenAI 兼容（Cline、Cherry Studio、OpenCode 等）**
```
Base URL: http://127.0.0.1:8765/v1
API key:  任意内容
Model:    mimo-pro
```

**Anthropic 兼容（Claude Code）**
```
ANTHROPIC_BASE_URL=http://127.0.0.1:8765
ANTHROPIC_API_KEY=任意内容
```

**OpenCode** — 添加到 `~/.opencode.json`：
```json
{
  "providers": {
    "xiaomi": {
      "type": "openai",
      "api_base": "http://127.0.0.1:8765/v1",
      "api_key": "sk-xiaomi-miclaw",
      "models": [
        { "name": "mimo-pro", "max_input_tokens": 256000 },
        { "name": "xiaomi/mimo", "max_input_tokens": 256000 }
      ]
    }
  },
  "default_provider": "xiaomi",
  "default_model": "mimo-pro"
}
```
然后运行 `opencode --provider xiaomi --model mimo-pro`。

## 构建

要求：Rust 1.77+、Node.js 20+、pnpm 9+。

```bash
git clone <仓库地址> && cd mix_api_bridge
pnpm install && pnpm build
cd src-tauri && cargo build --release --bin mix_api_bridge
```

## FAQ

**这是 miclaw 的分支吗？** 不是。独立客户端，相同协议，无复制代码。

**需要安装官方 miclaw 吗？** 不需要。但账号需有 miclaw 权限。

**凭据存在哪？** 系统密钥环（macOS/Win/Linux）或 Docker 的 `/data` 卷。

## 许可证

[MIT](LICENSE)

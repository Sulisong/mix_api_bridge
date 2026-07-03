<div align="center">

# mix_api_bridge

[![EN](https://img.shields.io/badge/EN-English-blue?style=for-the-badge)](#english)
[![ZH](https://img.shields.io/badge/ZH-%E4%B8%AD%E6%96%87%E6%96%87%E6%A1%A3-orange?style=for-the-badge)](#chinese)

</div>

---

<h1 id="english">English</h1>

<div align="center">

**Unified proxy for Xiaomi Mimo (miclaw) models and OpenCode free models.**

Expose both as OpenAI / Anthropic-compatible local endpoints. Mimo models require a miclaw-permissioned Xiaomi account; OpenCode free models need no authentication.

[![Rust](https://img.shields.io/badge/Rust-1.77+-orange?logo=rust)](https://www.rust-lang.org/)
[![Vue 3](https://img.shields.io/badge/Vue-3-42b883?logo=vue.js)](https://vuejs.org/)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)]()
[![License](https://img.shields.io/badge/license-MIT-green)](#license)

</div>

---

## What it does

mix_api_bridge provides two independent model backends under one local API:

### 1. Mimo (Xiaomi miclaw)

Authenticates via Xiaomi OAuth, then proxies the official Mimo LLM family. Requires a Xiaomi account with miclaw access.

| Model id | Notes |
|---|---|
| `xiaomi/mimo` | Multimodal, 256 K context (default) |
| `xiaomi/mimo-pro` | Reasoning model with `thinking` traces, 256 K context |
| `xiaomi/mimo-claw-0301` | Claw 0301 reasoning snapshot |
| `xiaomi/MiniMax-M2.5` | MiniMax M2.5, 128 K context |
| `xiaomi/kimi-k2.5` | Kimi K2.5 reasoning, 128 K context |
| `xiaomi/glm-5` | GLM-5, 128 K context |
| `mimo-omni` | Alias to `xiaomi/mimo` |
| `mimo-pro` | Alias to `xiaomi/mimo-pro` |

### 2. OpenCode Free Models

Routes to `opencode.ai/zen/v1/chat/completions`. **No authentication required.** All OpenAI Chat Completions compatible.

| Model id | Notes |
|---|---|
| `deepseek-v4-flash-free` | DeepSeek V4 Flash |
| `big-pickle` | Claude/Anthropic-style chat |
| `nemotron-3-super-free` | NVIDIA Nemotron Super |
| `nemotron-3-ultra-free` | NVIDIA Nemotron Ultra |
| `mimo-v2.5-free` | Xiaomi Mimo v2.5 (free tier) |
| `minimax-m3-free` | MiniMax M3 |

## Features

- **Mimo: Xiaomi OAuth** - username + password + SMS/email 2FA
- **Auto token refresh** - `serviceToken` rotated transparently on 401
- **Keychain-backed storage** - never on disk in plaintext
- **OpenCode: zero auth** - free models, no login or API key needed
- **Dual backend, single API** - both backends via `/v1/chat/completions`
- **Optional HTTPS** - TLS with auto self-signed or custom PEM
- **Admin password** - Argon2-hashed WebUI control plane
- **API-key auth** - optional Bearer auth on `/v1`
- **Per-model token usage** - windowed charts (1h/1d/7d/30d)
- **Browser WebUI** - full console at `http://127.0.0.1:8765`
- **Live request log** - real-time SSE log in WebUI
- **Headless deployment** - server binary for non-graphical machines
- **Optional desktop tray**

## Quick start

```bash
./mix_api_bridge server
# Open http://127.0.0.1:8765
```

For Mimo models, sign in with your Xiaomi account in the WebUI. OpenCode free models work immediately - just pick a model.

### Client configuration

All routes go through `http://127.0.0.1:8765/v1`. Switch between model families by changing the `model` field.

**OpenAI-compatible (Cline, Cherry Studio, OpenCode, ...)**
```
Base URL: http://127.0.0.1:8765/v1
API key:  anything (unless admin-enforced)
Model:    mimo-pro                 # Mimo models
          deepseek-v4-flash-free   # OpenCode free models
```

**Anthropic-compatible (Claude Code)**
```
ANTHROPIC_BASE_URL=http://127.0.0.1:8765
ANTHROPIC_API_KEY=anything
Model: mimo-pro
```

**curl examples**
```bash
# Mimo model
curl -N http://127.0.0.1:8765/v1/chat/completions \\
  -H 'content-type: application/json' \\
  -d '{"model":"mimo-pro","stream":true,"messages":[{"role":"user","content":"hi"}]}'

# OpenCode free model (no auth needed)
curl -N http://127.0.0.1:8765/v1/chat/completions \\
  -H 'content-type: application/json' \\
  -d '{"model":"deepseek-v4-flash-free","stream":true,"messages":[{"role":"user","content":"hi"}]}'
```

## Build from source

Prerequisites: Rust 1.77+, Node.js 20+, pnpm 9+.

```bash
git clone <this-repo> && cd mix_api_bridge
pnpm install && pnpm build
cd src-tauri && cargo build --release --bin mix_api_bridge
```

Binaries land in `src-tauri/target/release/`.

## Architecture

```
                     /v1/chat/completions
                +---------------------------+
                |  mimo models              |
                |    -> api.miclaw.xiaomi.net|
                |  opencode models          |
                |    -> opencode.ai/zen      |
                +---------------------------+
```

## FAQ

**Is this a fork of miclaw?** No - independent client, same protocol, no copied code.

**Need the official miclaw app?** No, but Mimo models require a Xiaomi account with miclaw access.

**OpenCode models need auth?** No - they hit `opencode.ai/zen/v1/chat/completions` directly, no login required.

**Where are credentials stored?** OS keyring (macOS/Win/Linux) or `/data` volume in Docker.

## License

[MIT](LICENSE)

---

<h1 id="chinese">中文</h1>

<div align="center">

**小米 Mimo（miclaw）模型与 OpenCode 免费模型的统一本地代理。**

将两种模型后端统一暴露为 OpenAI / Anthropic 兼容的本地端点。Mimo 模型需要已获 miclaw 权限的小米账号；OpenCode 免费模型无需任何认证。

[![Rust](https://img.shields.io/badge/Rust-1.77+-orange?logo=rust)](https://www.rust-lang.org/)
[![Vue 3](https://img.shields.io/badge/Vue-3-42b883?logo=vue.js)](https://vuejs.org/)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)]()
[![License](https://img.shields.io/badge/license-MIT-green)](#license)

</div>

---

## 它能做什么

mix_api_bridge 提供两个独立的模型后端，通过统一的本地 API 访问：

### 1. Mimo（小米 miclaw）

通过小米 OAuth 认证，代理官方 Mimo 系列 LLM。需要已获 miclaw 权限的小米账号。

| Model id | 说明 |
|---|---|
| `xiaomi/mimo` | 多模态，256K 上下文（默认） |
| `xiaomi/mimo-pro` | 推理模型，256K 上下文 |
| `xiaomi/mimo-claw-0301` | Claw 0301 推理快照 |
| `xiaomi/MiniMax-M2.5` | MiniMax M2.5，128K |
| `xiaomi/kimi-k2.5` | Kimi K2.5 推理，128K |
| `xiaomi/glm-5` | GLM-5，128K |
| `mimo-omni` | 别名 -> `xiaomi/mimo` |
| `mimo-pro` | 别名 -> `xiaomi/mimo-pro` |

### 2. OpenCode 免费模型

路由到 `opencode.ai/zen/v1/chat/completions`。**无需任何认证**，即开即用。

| Model id | 说明 |
|---|---|
| `deepseek-v4-flash-free` | DeepSeek V4 Flash |
| `big-pickle` | Claude/Anthropic 风格对话 |
| `nemotron-3-super-free` | NVIDIA Nemotron Super |
| `nemotron-3-ultra-free` | NVIDIA Nemotron Ultra |
| `mimo-v2.5-free` | 小米 Mimo v2.5（免费版） |
| `minimax-m3-free` | MiniMax M3 |

## 功能特性

- **Mimo：小米 OAuth** - 用户名 + 密码 + 短信/邮箱双因素认证
- **自动令牌刷新** - 401 时透明自动轮换
- **密钥环存储** - 密钥环加密，永不磁盘明文
- **OpenCode：零认证** - 免费模型即开即用，无需登录
- **双后端，统一 API** - 通过 `/v1/chat/completions` 切换模型
- **可选 HTTPS** - 自签名或自定义证书
- **管理员密码** - Argon2 加密 + Cookie 会话
- **API Key 认证** - 可选 Bearer 认证
- **逐模型 Token 统计** - 仪表盘图表
- **浏览器 WebUI** - `http://127.0.0.1:8765`
- **实时请求日志** - SSE 实时流
- **无头部署** - 无需图形界面
- **可选桌面托盘**

## 快速开始

```bash
./mix_api_bridge server
# 浏览器打开 http://127.0.0.1:8765
```

Mimo 模型需在 WebUI 中登录小米账号。OpenCode 免费模型即开即用，直接选模型即可。

### 客户端配置

所有路由统一走 `http://127.0.0.1:8765/v1`，改 `model` 字段切换模型族。

**OpenAI 兼容（Cline、Cherry Studio、OpenCode 等）**
```
Base URL: http://127.0.0.1:8765/v1
API key:  任意内容（除非管理员开启了 API Key 认证）
Model:    mimo-pro                   # Mimo 模型
          deepseek-v4-flash-free     # OpenCode 免费模型
```

**Anthropic 兼容（Claude Code）**
```
ANTHROPIC_BASE_URL=http://127.0.0.1:8765
ANTHROPIC_API_KEY=任意内容
Model: mimo-pro
```

**curl 示例**
```bash
# Mimo 模型
curl -N http://127.0.0.1:8765/v1/chat/completions \\
  -H 'content-type: application/json' \\
  -d '{"model":"mimo-pro","stream":true,"messages":[{"role":"user","content":"你好"}]}'

# OpenCode 免费模型（无需认证）
curl -N http://127.0.0.1:8765/v1/chat/completions \\
  -H 'content-type: application/json' \\
  -d '{"model":"deepseek-v4-flash-free","stream":true,"messages":[{"role":"user","content":"你好"}]}'
```

## 构建

要求：Rust 1.77+、Node.js 20+、pnpm 9+。

```bash
git clone <仓库地址> && cd mix_api_bridge
pnpm install && pnpm build
cd src-tauri && cargo build --release --bin mix_api_bridge
```

二进制文件输出到 `src-tauri/target/release/`。

## 架构

```
                     /v1/chat/completions
                +---------------------------+
                |  mimo 模型                |
                |    -> api.miclaw.xiaomi.net|
                |  opencode 模型            |
                |    -> opencode.ai/zen      |
                +---------------------------+
```

## FAQ

**这是 miclaw 的分支吗？** 不是。独立客户端，相同协议，无复制代码。

**需要安装官方 miclaw 吗？** 不需要。但 Mimo 模型需有 miclaw 权限的小米账号。

**OpenCode 免费模型需要认证吗？** 不需要。直接请求 `opencode.ai/zen/v1/chat/completions`，无需登录。

**凭据存在哪？** 系统密钥环（macOS/Win/Linux）或 Docker 的 `/data` 卷。

## 许可证

[MIT](LICENSE)

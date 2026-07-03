<div align="center">

# mix_api_bridge

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

> ⚠️ **Account requirement**: your Xiaomi account must already be approved for miclaw access. If the WebUI shows "需要 miclaw 内测权限" or the proxy returns 401 right after login, the account isn't allowlisted — apply through the official miclaw channel first.

Eight model ids are exposed, all routed through the official Xiaomi PC channel. The first six mirror the current miclaw cloud registry; the last two are short back-compat aliases:

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

- 🔐 **Real Xiaomi OAuth** — username + password + SMS / email 2FA, exactly like the desktop client
- 🔄 **Auto token refresh** — `serviceToken` rotated transparently on 401
- 🔑 **Keychain-backed storage** — credentials live in macOS Keychain / Windows DPAPI / Linux Secret Service, never on disk in plaintext
- 🔌 **Two protocols, one bridge** — speaks both OpenAI Chat Completions and Anthropic Messages
- 🔒 **Optional HTTPS** — serve the WebUI + proxy over TLS; a self-signed cert is auto-generated, or bring your own PEM
- 🛡 **Admin password** — a first-run setup guards the WebUI control plane (`/api/*`) behind an Argon2-hashed password and cookie session
- 🪪 **API-key auth** — optionally require `Authorization: Bearer` keys on `/v1`; create and revoke them from the dashboard
- 📊 **Per-model usage** — token accounting per model, with windowed (`1h` / `1d` / `7d` / `30d`) charts in the dashboard
- 🌐 **Browser WebUI** — the full management console at `http://127.0.0.1:8765`
- 📡 **Live request log** — WebUI streams every proxy hit in real time
- 🧩 **Headless deployment** — server binary runs on machines without a graphical session
- 🖥 **Optional desktop tray** — no embedded webview window; tray menu only opens WebUI or exits
- 🤖 **OpenCode-ready** — seamless integration with OpenCode's OpenAI-compatible endpoint

## Quick start

### From a release

1. Grab the binary archive for your platform from the [Releases](../../releases) page.
2. Start the headless server:

   ```bash
   ./mix_api_bridge server
   ```

3. Open `http://127.0.0.1:8765` in a browser and sign in with your miclaw-permissioned Xiaomi account.
4. OpenAI / Responses / Anthropic / OpenCode endpoints are available immediately on the same port.

Desktop users can launch `mix_api_bridge_desktop` instead. It starts the same local service, opens the WebUI in your default browser, and adds a tray icon with **打开webui** / **退出**.
The tray also exposes a **启用 HTTPS / 关闭 HTTPS** toggle.

For remote/headless servers, keep the default localhost binding and use an SSH tunnel:

```bash
ssh -L 8765:127.0.0.1:8765 user@server
```

### Client configuration

**OpenAI-compatible (Cline, Cherry Studio, OpenAI SDK, …)**

```
Base URL: http://127.0.0.1:8765/v1
API key:  anything
Model:    mimo-pro      # or any model from /v1/models
```

**Anthropic-compatible (Claude Code, etc.)**

```
ANTHROPIC_BASE_URL=http://127.0.0.1:8765
ANTHROPIC_API_KEY=anything
Model: mimo-pro
```

**OpenCode**

Add an OpenAI-compatible provider to `~/.opencode.json`:

```json
{
  "providers": {
    "xiaomi": {
      "type": "openai",
      "api_base": "http://127.0.0.1:8765/v1",
      "api_key": "sk-xiaomi-miclaw",
      "models": [
        { "name": "mimo-pro", "max_input_tokens": 256000 },
        { "name": "xiaomi/mimo", "max_input_tokens": 256000 },
        { "name": "xiaomi/kimi-k2.5", "max_input_tokens": 128000 },
        { "name": "xiaomi/glm-5", "max_input_tokens": 128000 }
      ]
    }
  },
  "default_provider": "xiaomi",
  "default_model": "mimo-pro"
}
```

Then launch OpenCode with:

```bash
opencode --provider xiaomi --model mimo-pro
```

Or switch during a session with `/provider xiaomi` and `/model mimo-pro`.

**curl smoke test**

```bash
# OpenAI
curl -N http://127.0.0.1:8765/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{"model":"mimo-pro","stream":true,"messages":[{"role":"user","content":"hi"}]}'

# Anthropic
curl -N http://127.0.0.1:8765/v1/messages \
  -H 'content-type: application/json' \
  -H 'anthropic-version: 2023-06-01' \
  -d '{"model":"mimo-pro","max_tokens":256,"stream":true,"messages":[{"role":"user","content":"hi"}]}'
```

## Build from source

Prerequisites: Rust 1.77+, Node.js 20+, pnpm 9+.

```bash
git clone <this-repo> mix_api_bridge && cd mix_api_bridge
pnpm install
pnpm build              # build the browser WebUI into dist/
cd src-tauri
cargo build --release --bin mix_api_bridge
cargo build --release --features desktop --bin mix_api_bridge_desktop
```

Binaries end up in `src-tauri/target/release/`.

## Architecture

```
┌─────────────────┐
│ Browser WebUI   │     /api/*     ┌──────────────────────────┐
│ Login / Status  │◄──────────────►│ Rust headless server     │
│ Logs panel      │    SSE logs    │ optional desktop tray    │
└─────────────────┘                └────────────┬─────────────┘
                                                │ axum on 127.0.0.1:8765
                                                ▼
                       ┌────────────────────────────────────────────┐
                       │ /v1/chat/completions  OpenAI passthrough   │
                       │ /v1/responses         OpenAI compat layer  │
                       │ /v1/messages          Anthropic ⇆ OpenAI   │
                       │ /v1/models            Static manifest      │
                       └────────────────────────┬───────────────────┘
                                                │
                                                ▼
                          api.miclaw.xiaomi.net /osbot/pc/llm/v1/...
                          (Cookie: serviceToken+cUserId, UA: node)
```

## FAQ

**Is this a fork of miclaw?**
No — mix_api_bridge is an independent client that speaks the same protocol. No code is copied from the official client.

**Does it work without the official miclaw app installed?**
Yes. mix_api_bridge talks directly to Xiaomi's account and inference endpoints; the desktop client doesn't need to be installed. **Your account does, however, need miclaw access** — without it the inference API returns 401 even with a valid serviceToken.

**Where are my credentials stored?**
Encrypted in your OS keyring (macOS Keychain / Windows DPAPI / Linux Secret Service). Only the session blob — `passToken / serviceToken / userId / cUserId / ssecurity / nick` — is kept; your password is never persisted. Docker sets `MICLAW_API_BRIDGE_DISABLE_KEYRING=1`, so the session blob is stored in the mounted `/data` volume instead.

## Disclaimer

This project is an independent reverse-engineering effort intended for **educational and personal use**. It is not affiliated with, endorsed by, or sponsored by Xiaomi. By using mix_api_bridge you accept full responsibility for compliance with the Xiaomi terms of service applicable to your account. The authors provide no warranty and accept no liability.

## License

[MIT](LICENSE)

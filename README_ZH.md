<div align="center">

# mix_api_bridge

**将小米 miclaw（米络）的模型作为 OpenAI / Anthropic / OpenCode 兼容的本地端点运行。**

使用已获得 miclaw 权限的小米账号登录，然后在浏览器、OpenAI 客户端、Claude 客户端或 **OpenCode** 中访问 `http://127.0.0.1:8765`。

[![Rust](https://img.shields.io/badge/Rust-1.77+-orange?logo=rust)](https://www.rust-lang.org/)
[![Vue 3](https://img.shields.io/badge/Vue-3-42b883?logo=vue.js)](https://vuejs.org/)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)]()
[![License](https://img.shields.io/badge/license-MIT-green)](#license)

</div>

---

## 它能做什么

mix_api_bridge 通过与官方 **小米 miclaw** 桌面客户端相同的 OAuth 流程登录你的小米账号，然后将 `mimo` 系列 LLM 以本地 HTTP 端点形式暴露出来：

- `POST /v1/chat/completions` — OpenAI Chat Completions（可直接用于 Cline、Cherry Studio、OpenAI SDK、**OpenCode** 等）
- `POST /v1/responses` — OpenAI Responses API（原生透传，不可用时回退至 Chat Completions）
- `POST /v1/messages` — Anthropic Messages，完整的 SSE 事件转换（可直接用于 Claude Code 及任何支持 `ANTHROPIC_BASE_URL` 的客户端）
- `GET /v1/models` — 八个已验证的模型 ID

> ⚠️ **账号要求**：你的小米账号必须已获得 miclaw 内测权限。如果 WebUI 提示"需要 miclaw 内测权限"或登录后代理返回 401，说明账号尚未加入白名单——请先通过官方 miclaw 渠道申请。

开放八个模型 ID，全部通过官方小米 PC 通道路由。前六个对应当前的 miclaw 云端注册表，后两个是向后兼容的短别名：

| Model id | 说明 |
|---|---|
| `xiaomi/mimo` | 多模态模型，256K 上下文（默认） |
| `xiaomi/mimo-pro` | 推理模型，含 `thinking` 轨迹，256K 上下文 |
| `xiaomi/mimo-claw-0301` | Claw 0301 推理快照 |
| `xiaomi/MiniMax-M2.5` | MiniMax M2.5，128K 上下文 |
| `xiaomi/kimi-k2.5` | Kimi K2.5 推理，128K 上下文 |
| `xiaomi/glm-5` | GLM-5，128K 上下文 |
| `mimo-omni` | 别名 → `xiaomi/mimo` |
| `mimo-pro` | 别名 → `xiaomi/mimo-pro` |

## 功能特性

- 🔐 **真实小米 OAuth** — 用户名 + 密码 + 短信/邮箱双因素认证，与桌面客户端完全相同
- 🔄 **自动令牌刷新** — `serviceToken` 在收到 401 时透明地自动轮换
- 🔑 **密钥环存储** — 凭据保存在 macOS Keychain / Windows DPAPI / Linux Secret Service 中，从不以明文写入磁盘
- 🔌 **双协议合一桥接** — 同时兼容 OpenAI Chat Completions 和 Anthropic Messages
- 🔒 **可选 HTTPS** — 为 WebUI + 代理提供 TLS 支持；自动生成自签名证书，或使用自定义 PEM
- 🛡 **管理员密码** — 首次运行设置，WebUI 控制面（`/api/*`）受 Argon2 哈希密码和 Cookie 会话保护
- 🪪 **API Key 认证** — 可选在 `/v1` 上要求 `Authorization: Bearer` 密钥；密钥从仪表盘创建和撤销（仅存储哈希 + 显示前缀）
- 📊 **逐模型用量统计** — 按模型统计 Token，支持窗口期（1小时/1天/7天/30天）仪表盘图表
- 🌐 **浏览器 WebUI** — 在 `http://127.0.0.1:8765` 提供完整界面的控制台
- 📡 **实时请求日志** — WebUI 实时流式显示每一次代理请求
- 🧩 **无头部署** — 服务端二进制可在无图形界面的机器上运行
- 🖥 **可选桌面托盘** — 无需嵌入式 WebView 窗口；托盘菜单仅用于打开 WebUI 或退出
- 🤖 **OpenCode 原生支持** — 与 OpenCode 的 OpenAI 兼容端点无缝集成

## 快速开始

### 从 Release 开始

1. 从 [Releases](../../releases) 页面下载对应平台的二进制压缩包。
2. 启动无头服务端：

   ```bash
   ./mix_api_bridge server
   ```

3. 在浏览器中打开 `http://127.0.0.1:8765`，用已获得 miclaw 权限的小米账号登录。
4. OpenAI / Responses / Anthropic / OpenCode 端点立即可用，端口相同。

桌面用户可启动 `mix_api_bridge_desktop` 代替。它启动同样的本地服务，在默认浏览器中打开 WebUI，并添加托盘图标（**打开webui** / **退出**）。
托盘还提供 **启用 HTTPS / 关闭 HTTPS** 切换，运行时翻转 TLS 并重启服务。

远程/无头服务器建议保持默认的 localhost 绑定，使用 SSH 隧道：

```bash
ssh -L 8765:127.0.0.1:8765 user@server
```

### 配置客户端

**OpenAI 兼容（Cline、Cherry Studio、OpenAI SDK、…）**

```
Base URL: http://127.0.0.1:8765/v1
API key:  任意内容
Model:    mimo-pro      # 或 /v1/models 中的任一模型
```

**Anthropic 兼容（Claude Code 等）**

```
ANTHROPIC_BASE_URL=http://127.0.0.1:8765
ANTHROPIC_API_KEY=任意内容
Model: mimo-pro
```

**OpenCode 配置**

在 `~/.opencode.json` 中添加 OpenAI 兼容提供商：

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

然后在终端中运行：

```bash
opencode --provider xiaomi --model mimo-pro
```

或在 OpenCode 会话内通过 `/provider xiaomi` 和 `/model mimo-pro` 切换。

**curl 快速测试**

```bash
# OpenAI
curl -N http://127.0.0.1:8765/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{"model":"mimo-pro","stream":true,"messages":[{"role":"user","content":"你好"}]}'

# Anthropic
curl -N http://127.0.0.1:8765/v1/messages \
  -H 'content-type: application/json' \
  -H 'anthropic-version: 2023-06-01' \
  -d '{"model":"mimo-pro","max_tokens":256,"stream":true,"messages":[{"role":"user","content":"你好"}]}'
```

## 构建

前置条件：Rust 1.77+、Node.js 20+、pnpm 9+。

```bash
git clone <仓库地址> mix_api_bridge && cd mix_api_bridge
pnpm install
pnpm build                      # 构建浏览器 WebUI -> dist/
cd src-tauri
cargo build --release --bin mix_api_bridge
cargo build --release --features desktop --bin mix_api_bridge_desktop
```

二进制文件输出到 `src-tauri/target/release/`。

## 架构

```
┌─────────────────┐
│ 浏览器 WebUI     │     /api/*     ┌──────────────────────────┐
│ 登录 / 状态      │◄──────────────►│ Rust 无头服务端           │
│ 日志面板         │    SSE 日志    │ 可选桌面托盘              │
└─────────────────┘                └────────────┬─────────────┘
                                                │ axum on 127.0.0.1:8765
                                                ▼
                       ┌────────────────────────────────────────────┐
                       │ /v1/chat/completions  OpenAI 透传          │
                       │ /v1/responses         OpenAI 兼容层         │
                       │ /v1/messages          Anthropic ⇆ OpenAI   │
                       │ /v1/models            静态模型清单           │
                       └────────────────────────┬───────────────────┘
                                                │
                                                ▼
                          api.miclaw.xiaomi.net /osbot/pc/llm/v1/...
                          (Cookie: serviceToken+cUserId, UA: node)
```

## FAQ

**这是 miclaw 的分支吗？**
不——mix_api_bridge 是一个独立的客户端，使用相同的协议通信。不从官方客户端复制任何代码。

**需要安装官方 miclaw 桌面客户端吗？**
不需要。mix_api_bridge 直接与小米的账号和推理端点通信。**但你的账号需要 miclaw 权限**——否则即使有有效的 serviceToken，推理 API 也会返回 401。

**凭据存储在哪里？**
加密存储在系统密钥环中（macOS Keychain / Windows DPAPI / Linux Secret Service）。仅保存会话数据（`passToken / serviceToken / userId / cUserId / ssecurity / nick`），密码从不持久化。Docker 环境中会话数据存储在挂载的 `/data` 卷中。

## 免责声明

本项目是独立的逆向工程成果，仅供 **教育和个人使用**。与小米公司无任何关联、背书或赞助关系。使用 mix_api_bridge 即表示您自行承担遵守小米服务条款的全部责任。作者不提供任何担保，也不承担任何责任。

## 许可证

[MIT](LICENSE)

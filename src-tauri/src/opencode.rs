//! HTTP client for OpenCode free model API.
//!
//! Unlike Mimo, OpenCode free models require no authentication. The client
//! simply POSTs to `opencode.ai/zen/v1/chat/completions` and streams the
//! response. All models listed here have been verified against that endpoint.

use crate::error::Result;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// OpenCode upstream base URL.
pub const OPENCODE_HOST: &str = "https://opencode.ai";

/// Chat Completions path — the single endpoint used by all OpenCode free
/// models. The upstream outputs either OpenAI SSE or Anthropic SSE depending
/// on the model (e.g. `big-pickle` returns Anthropic-style SSE).
pub const PATH_CHAT: &str = "/zen/v1/chat/completions";

/// Known OpenCode free models.
pub const KNOWN_MODELS: &[(&str, &str, &str)] = &[
    ("deepseek-v4-flash-free", "opencode", "chat (DeepSeek V4 Flash)"),
    ("big-pickle", "opencode", "chat (Claude/Anthropic)"),
    ("nemotron-3-super-free", "opencode", "chat (NVIDIA Nemotron Super)"),
    ("nemotron-3-ultra-free", "opencode", "chat (NVIDIA Nemotron Ultra)"),
    ("mimo-v2.5-free", "opencode", "chat (Xiaomi Mimo)"),
    ("minimax-m3-free", "opencode", "chat (MiniMax M3)"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub owned_by: String,
    pub family: String,
}

pub fn known_models() -> Vec<ModelInfo> {
    KNOWN_MODELS
        .iter()
        .map(|(id, owned_by, family)| ModelInfo {
            id: id.to_string(),
            object: "model".into(),
            owned_by: owned_by.to_string(),
            family: family.to_string(),
        })
        .collect()
}

/// Returns `true` if the model id is an OpenCode free model.
pub fn is_opencode_model(model: &str) -> bool {
    KNOWN_MODELS.iter().any(|(id, _, _)| *id == model)
}

/// Lightweight HTTP client for OpenCode's free model API.
pub struct OpenCodeClient {
    client: reqwest::Client,
}

impl OpenCodeClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("user-agent"),
            HeaderValue::from_static("mix-api-bridge/0.1.0"),
        );
        headers.insert(
            HeaderName::from_static("accept"),
            HeaderValue::from_static("*/*"),
        );
        headers.insert(
            HeaderName::from_static("accept-encoding"),
            HeaderValue::from_static("gzip"),
        );
        Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .gzip(true)
                .pool_idle_timeout(std::time::Duration::from_secs(30))
                .pool_max_idle_per_host(16)
                .build()
                .expect("OpenCodeClient: reqwest::Client::new"),
        }
    }

    /// POST a JSON body to the OpenCode Chat Completions endpoint and return
    /// the upstream response (body still unread / streaming).
    pub async fn post_json(&self, body: Value) -> Result<reqwest::Response> {
        let resp = self
            .client
            .post(format!("{OPENCODE_HOST}{PATH_CHAT}"))
            .json(&body)
            .send()
            .await?;
        Ok(resp)
    }
}

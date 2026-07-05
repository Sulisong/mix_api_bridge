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

    /// POST through the proxy pool. Acquires a proxy node, attaches it to the
    /// request, and returns the response along with the node id so the caller
    /// can mark success/failure after consuming the stream.
    pub async fn post_json_proxied(
        &self,
        body: Value,
        pool: &crate::proxy_pool::ProxyPoolStore,
    ) -> std::result::Result<
        (reqwest::Response, Option<String>),
        crate::error::BridgeError,
    > {
        let mode = pool.proxy_mode().await;
        let lease = pool.acquire().await;

        // Proxy required but none available → hard error
        if mode == "required" && lease.is_none() {
            return Err(crate::error::BridgeError::Proxy(
                "Proxy is required but no proxy node is available".into(),
            ));
        }

        let node_id = lease.as_ref().map(|l| l.node.id.clone());
        let mut req = self
            .client
            .post(format!("{OPENCODE_HOST}{PATH_CHAT}"))
            .json(&body);

        // Attach proxy agent if we have one
        if let Some(ref lease) = lease {
            if let Some(proxy) = crate::proxy_pool::ProxyPoolStore::to_http_proxy(&lease.node) {
                req = req.proxy(proxy);
            }
        }

        let resp = req.send().await.map_err(|e| {
            // Network error — mark proxy failure
            if let Some(ref nid) = node_id {
                let node_id = nid.clone();
                let pool2 = pool;
                tokio::spawn(async move {
                    pool2.mark_failure(&node_id, &e.to_string(), 502).await;
                });
            }
            crate::error::BridgeError::Proxy(e.to_string())
        })?;

        // Initial response — mark based on status
        let status = resp.status();
        if let Some(ref nid) = node_id {
            let nid = nid.clone();
            let pool2 = pool;
            if status.as_u16() == 429 {
                pool2.mark_failure(&nid, "Upstream returned 429", 429).await;
            } else if !status.is_success() {
                pool2
                    .mark_failure(&nid, &format!("Upstream returned {status}"), status.as_u16())
                    .await;
            } else {
                pool2.mark_success(&nid).await;
            }
        }

        Ok((resp, node_id))
    }
}

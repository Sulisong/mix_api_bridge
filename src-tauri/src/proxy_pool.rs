//! Proxy pool — manage a pool of outbound proxy nodes.
//!
//! Ported from OpenCodeProxyHub (TypeScript) to Rust.
//! Features: priority-fill, concurrency caps, cooldowns, auto-disable on 429.

use crate::storage::Storage;
use chrono::{DateTime, Utc};
use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// ── Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Http,
    Https,
    Socks5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRequestResult {
    pub at: String,
    pub ok: bool,
    pub status_code: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: ProxyType,
    pub url: String,
    pub enabled: bool,
    pub weight: u32,
    pub max_concurrency: u32,
    #[serde(default)]
    pub current_concurrency: u32,
    pub daily_request_limit: u64,
    #[serde(default)]
    pub daily_request_count: u64,
    #[serde(default)]
    pub daily_count_date: String,
    #[serde(default)]
    pub auto_disable_when_daily_limit_reached: bool,
    #[serde(default)]
    pub consecutive_rate_limit_count: u32,
    pub cooldown_until: Option<String>,
    #[serde(default)]
    pub success_count: u64,
    #[serde(default)]
    pub fail_count: u64,
    #[serde(default)]
    pub recent_results: Vec<ProxyRequestResult>,
    pub last_error: Option<String>,
    pub last_used_at: Option<String>,
    pub last_checked_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProxyLease {
    pub node: ProxyNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProxyFile {
    version: u32,
    proxies: Vec<ProxyNode>,
}

// ── Proxy Pool Store ─────────────────────────────────────────────────

pub struct ProxyPoolStore {
    inner: Arc<Mutex<ProxyPoolInner>>,
    storage: Arc<Storage>,
}

struct ProxyPoolInner {
    proxies: Vec<ProxyNode>,
}

impl ProxyPoolStore {
    pub fn new(storage: Arc<Storage>) -> Self {
        let this = Self {
            inner: Arc::new(Mutex::new(ProxyPoolInner { proxies: vec![] })),
            storage,
        };
        let _ = this.load();
        this
    }

    fn load(&self) -> crate::error::Result<()> {
        let data: Option<ProxyFile> = self.storage.load_blob("proxy_pool")?;
        if let Some(file) = data {
            let proxies = file
                .proxies
                .into_iter()
                .map(|mut n| {
                    n.current_concurrency = 0;
                    n
                })
                .collect();
            let mut inner = self.inner.blocking_lock();
            inner.proxies = proxies;
        }
        Ok(())
    }

    fn persist(&self) {
        let inner = self.inner.blocking_lock();
        let file = ProxyFile {
            version: 1,
            proxies: inner.proxies.clone(),
        };
        let _ = self.storage.save_blob("proxy_pool", &file);
    }

    fn today() -> String {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    }

    fn reset_daily_if_needed(proxies: &mut [ProxyNode]) {
        let today = Self::today();
        for node in proxies.iter_mut() {
            if node.daily_count_date == today {
                continue;
            }
            node.daily_count_date = today.clone();
            node.daily_request_count = 0;
            if node.auto_disable_when_daily_limit_reached
                && node.last_error.as_deref() == Some("Daily request limit reached")
            {
                node.enabled = true;
                node.last_error = None;
            }
        }
    }

    // ── Public API ───────────────────────────────────────────────────

    /// Snapshot of all proxy nodes.
    pub async fn list(&self) -> Vec<ProxyNode> {
        let mut inner = self.inner.lock().await;
        Self::reset_daily_if_needed(&mut inner.proxies);
        inner.proxies.clone()
    }

    pub async fn create(&self, input: ProxyNodeInput) -> ProxyNode {
        let mut inner = self.inner.lock().await;
        let node = input.into_node();
        inner.proxies.push(node.clone());
        self.persist();
        node
    }

    pub async fn update(&self, id: &str, input: ProxyNodeInput) -> Option<ProxyNode> {
        let mut inner = self.inner.lock().await;
        let node = inner.proxies.iter_mut().find(|n| n.id == id)?;
        if let Some(name) = input.name {
            node.name = name;
        }
        if let Some(t) = input.proxy_type {
            node.proxy_type = t;
        }
        if let Some(url) = input.url {
            node.url = url;
        }
        if let Some(enabled) = input.enabled {
            node.enabled = enabled;
            if enabled {
                node.consecutive_rate_limit_count = 0;
                if node.last_error.as_deref() == Some("Disabled after 5 consecutive 429 responses")
                {
                    node.last_error = None;
                }
            }
        }
        if let Some(w) = input.weight {
            node.weight = w.max(1);
        }
        if let Some(mc) = input.max_concurrency {
            node.max_concurrency = mc.max(1);
        }
        if let Some(dl) = input.daily_request_limit {
            node.daily_request_limit = dl;
        }
        if let Some(ad) = input.auto_disable_when_daily_limit_reached {
            node.auto_disable_when_daily_limit_reached = ad;
        }
        let clone = node.clone();
        self.persist();
        Some(clone)
    }

    pub async fn delete(&self, id: &str) -> bool {
        let mut inner = self.inner.lock().await;
        let before = inner.proxies.len();
        inner.proxies.retain(|n| n.id != id);
        if inner.proxies.len() == before {
            return false;
        }
        self.persist();
        true
    }

    /// Acquire the best available proxy node (priority-fill by weight).
    pub async fn acquire(&self) -> Option<ProxyLease> {
        let mut inner = self.inner.lock().await;
        Self::reset_daily_if_needed(&mut inner.proxies);

        let now = Utc::now().timestamp_millis();
        inner.proxies.sort_by(|a, b| b.weight.cmp(&a.weight));

        for node in inner.proxies.iter_mut() {
            if !node.enabled {
                continue;
            }
            if let Some(ref cd) = node.cooldown_until {
                if let Ok(dt) = cd.parse::<DateTime<Utc>>() {
                    if dt.timestamp_millis() > now {
                        continue;
                    }
                }
            }
            if node.daily_request_limit > 0 && node.daily_request_count >= node.daily_request_limit
            {
                continue;
            }
            if node.current_concurrency >= node.max_concurrency {
                continue;
            }

            node.current_concurrency += 1;
            node.daily_request_count += 1;
            node.last_used_at = Some(Utc::now().to_rfc3339());

            if node.daily_request_limit > 0 && node.daily_request_count >= node.daily_request_limit
            {
                if node.auto_disable_when_daily_limit_reached {
                    node.enabled = false;
                    node.last_error = Some("Daily request limit reached".into());
                }
            }

            self.persist();
            return Some(ProxyLease {
                node: node.clone(),
            });
        }

        None
    }

    /// Mark the proxy node as having returned a successful response.
    pub async fn mark_success(&self, node_id: &str) {
        let mut inner = self.inner.lock().await;
        if let Some(node) = inner.proxies.iter_mut().find(|n| n.id == node_id) {
            node.success_count += 1;
            node.consecutive_rate_limit_count = 0;
            node.recent_results.push(ProxyRequestResult {
                at: Utc::now().to_rfc3339(),
                ok: true,
                status_code: 200,
            });
            if node.recent_results.len() > 20 {
                node.recent_results.remove(0);
            }
            node.last_error = None;
            node.last_checked_at = Some(Utc::now().to_rfc3339());
            node.current_concurrency = node.current_concurrency.saturating_sub(1);
            self.persist();
        }
    }

    /// Mark a proxy request as failed with the given status.
    /// 429s increment consecutive counter; after 5 the node is disabled.
    pub async fn mark_failure(&self, node_id: &str, error: &str, status_code: u16) {
        let mut inner = self.inner.lock().await;
        if let Some(node) = inner.proxies.iter_mut().find(|n| n.id == node_id) {
            node.fail_count += 1;
            node.recent_results.push(ProxyRequestResult {
                at: Utc::now().to_rfc3339(),
                ok: false,
                status_code,
            });
            if node.recent_results.len() > 20 {
                node.recent_results.remove(0);
            }
            node.last_error = Some(error.to_string());
            node.last_checked_at = Some(Utc::now().to_rfc3339());

            if status_code == 429 {
                node.consecutive_rate_limit_count += 1;
                if node.consecutive_rate_limit_count >= 5 {
                    node.enabled = false;
                    node.last_error = Some("Disabled after 5 consecutive 429 responses".into());
                    node.cooldown_until = None;
                }
            } else {
                let cd = Utc::now()
                    + chrono::Duration::minutes(5);
                node.cooldown_until = Some(cd.to_rfc3339());
            }

            node.current_concurrency = node.current_concurrency.saturating_sub(1);
            self.persist();
        }
    }

    /// Build a `reqwest::Proxy` from a proxy node (HTTP/HTTPS only).
    /// SOCKS5 requires the `socks` feature on reqwest; returns None when
    /// that feature is absent.
    pub fn to_http_proxy(node: &ProxyNode) -> Option<Proxy> {
        match node.proxy_type {
            ProxyType::Http => Proxy::http(&node.url).ok(),
            ProxyType::Https => Proxy::https(&node.url).ok(),
            ProxyType::Socks5 => {
                // Without the `socks` feature, `Proxy::all` won't handle socks5.
                // Log and skip.
                tracing::warn!(target = "proxy_pool", "SOCKS5 proxy requires reqwest socks feature; skipping");
                None
            }
        }
    }

    /// Get the proxy mode setting (direct/optional/required).
    /// "direct"  → never use proxies (treat pool as empty)
    /// "optional" → use proxy if available, else direct
    /// "required" → fail if no proxy available
    pub async fn proxy_mode(&self) -> String {
        std::env::var("MIX_PROXY_MODE").unwrap_or_else(|_| "optional".into())
    }
}

// ── Input for create/update ──────────────────────────────────────────

#[derive(Debug, Default)]
pub struct ProxyNodeInput {
    pub name: Option<String>,
    pub proxy_type: Option<ProxyType>,
    pub url: Option<String>,
    pub enabled: Option<bool>,
    pub weight: Option<u32>,
    pub max_concurrency: Option<u32>,
    pub daily_request_limit: Option<u64>,
    pub auto_disable_when_daily_limit_reached: Option<bool>,
}

impl ProxyNodeInput {
    pub fn into_node(self) -> ProxyNode {
        let now = Utc::now();
        let today = now.format("%Y-%m-%d").to_string();
        ProxyNode {
            id: uuid::Uuid::new_v4().to_string(),
            name: self.name.unwrap_or_else(|| "Unnamed proxy".into()),
            proxy_type: self.proxy_type.unwrap_or(ProxyType::Http),
            url: self.url.unwrap_or_default(),
            enabled: self.enabled.unwrap_or(true),
            weight: self.weight.unwrap_or(1).max(1),
            max_concurrency: self.max_concurrency.unwrap_or(10).max(1),
            current_concurrency: 0,
            daily_request_limit: self.daily_request_limit.unwrap_or(0),
            daily_request_count: 0,
            daily_count_date: today,
            auto_disable_when_daily_limit_reached: self
                .auto_disable_when_daily_limit_reached
                .unwrap_or(false),
            consecutive_rate_limit_count: 0,
            cooldown_until: None,
            success_count: 0,
            fail_count: 0,
            recent_results: vec![],
            last_error: None,
            last_used_at: None,
            last_checked_at: Some(now.to_rfc3339()),
        }
    }
}

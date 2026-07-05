//! Local HTTP proxy exposing OpenAI Chat Completions and Anthropic Messages
//! compatible endpoints, routed to mimo PC or OpenCode free models.

pub(crate) mod anthropic;
pub(crate) mod openai;
mod transport;

pub use transport::emit_log;

use crate::mimo::MimoClient;
use crate::opencode::OpenCodeClient;
use crate::state::LogEmitter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct ProxyController {
    pub mimo: Arc<MimoClient>,
    pub opencode: Arc<OpenCodeClient>,
    pub emitter: LogEmitter,
    pub usage: Arc<crate::usage::UsageStore>,
    verbose: AtomicBool,
    semaphore: Arc<Semaphore>,
}

impl ProxyController {
    pub fn new(
        mimo: Arc<MimoClient>,
        opencode: Arc<OpenCodeClient>,
        emitter: LogEmitter,
        usage: Arc<crate::usage::UsageStore>,
    ) -> Self {
        Self {
            mimo,
            opencode,
            emitter,
            usage,
            verbose: AtomicBool::new(false),
            semaphore: Arc::new(Semaphore::new(65535)),
        }
    }

    /// Whether request logs should include the full request body (prompt).
    pub fn verbose(&self) -> bool {
        self.verbose.load(Ordering::Relaxed)
    }

    pub fn set_verbose(&self, on: bool) {
        self.verbose.store(on, Ordering::Relaxed);
    }

    /// Returns `true` if `model` should be routed to the OpenCode upstream
    /// instead of mimo.
    pub fn is_opencode_model(&self, model: &str) -> bool {
        crate::opencode::is_opencode_model(model)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProxySnapshot {
    pub running: bool,
    pub addr: Option<String>,
    pub port: u16,
    pub active_port: Option<u16>,
    pub restart_required: bool,
}

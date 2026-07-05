//! mix_api_bridge: bridge Xiaomi mimo into local OpenAI/Claude compatible APIs.

pub mod auth;
pub mod decode;
pub mod error;
pub mod mimo;
pub mod opencode;
pub mod proxy;
pub mod proxy_pool;
pub mod security;
pub mod server;
pub mod service;
pub mod state;
pub mod storage;
pub mod usage;

pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("info,mix_api_bridge_lib=debug")
            }),
        )
        .try_init();
}

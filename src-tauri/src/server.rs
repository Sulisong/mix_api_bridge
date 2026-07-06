use crate::auth::login::LoginRequest;
use crate::error::{BridgeError, Result};
use crate::service::{SendTicketRequest, SetPortRequest, VerifyTicketRequest};
use crate::state::BridgeState;
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode, Uri};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use axum_server::tls_rustls::RustlsConfig;
use axum_server::Handle;
use futures_util::stream::{self, Stream};
use rust_embed::RustEmbed;
use serde::Serialize;
use serde_json::json;
use std::borrow::Cow;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 8765,
        }
    }
}

pub struct HttpServer {
    pub addr: SocketAddr,
    pub tls: bool,
    handle: Option<Handle>,
    task: Option<tokio::task::JoinHandle<()>>,
    state: Arc<BridgeState>,
}

impl HttpServer {
    pub fn webui_url(&self) -> String {
        let scheme = if self.tls { "https" } else { "http" };
        format!("{scheme}://{}", self.addr)
    }

    pub fn shutdown(mut self) {
        if let Some(handle) = self.handle.take() {
            handle.graceful_shutdown(Some(Duration::from_secs(3)));
        }
        self.state.clear_bound_addr();
    }

    /// Stop immediately and wait for the serve task (and thus the listener) to
    /// finish, so a fresh server can bind the same port right away. Used when
    /// toggling TLS at runtime.
    pub async fn shutdown_and_wait(mut self) {
        if let Some(handle) = self.handle.take() {
            handle.shutdown();
        }
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
        self.state.clear_bound_addr();
    }
}

pub async fn start_http(state: Arc<BridgeState>, config: ServerConfig) -> Result<HttpServer> {
    // Install the ring CryptoProvider once. Required because axum-server is
    // built with `tls-rustls-no-provider`; harmless if already installed.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let addr = SocketAddr::new(config.host, config.port);
    let app = router(state.clone());
    let handle = Handle::new();
    let tls_enabled = state.storage.settings().tls_enabled;

    // Bind synchronously so we can surface "port in use" before returning.
    let std_listener = std::net::TcpListener::bind(addr)
        .map_err(|e| BridgeError::Proxy(format!("bind {addr}: {e}")))?;
    std_listener
        .set_nonblocking(true)
        .map_err(|e| BridgeError::Proxy(format!("listener: {e}")))?;
    let bound = std_listener.local_addr().unwrap_or(addr);
    state.set_bound_addr(bound);

    let task = if tls_enabled {
        let tls_config = load_or_make_tls(&state).await?;
        let handle2 = handle.clone();
        let state2 = state.clone();
        let make_service = app.into_make_service();
        tokio::spawn(async move {
            if let Err(e) = axum_server::from_tcp_rustls(std_listener, tls_config)
                .handle(handle2)
                .serve(make_service)
                .await
            {
                tracing::error!(target = "server", "https server failed: {e}");
            }
            state2.clear_bound_addr();
        })
    } else {
        let handle2 = handle.clone();
        let state2 = state.clone();
        let make_service = app.into_make_service();
        tokio::spawn(async move {
            if let Err(e) = axum_server::from_tcp(std_listener)
                .handle(handle2)
                .serve(make_service)
                .await
            {
                tracing::error!(target = "server", "http server failed: {e}");
            }
            state2.clear_bound_addr();
        })
    };

    Ok(HttpServer {
        addr: bound,
        tls: tls_enabled,
        handle: Some(handle),
        task: Some(task),
        state,
    })
}

/// Resolve the TLS config: use the configured cert/key pair if present,
/// otherwise generate (and cache) a self-signed cert in the config dir.
async fn load_or_make_tls(state: &Arc<BridgeState>) -> Result<RustlsConfig> {
    let settings = state.storage.settings();
    if let (Some(cert), Some(key)) = (
        settings.tls_cert_path.clone(),
        settings.tls_key_path.clone(),
    ) {
        return RustlsConfig::from_pem_file(&cert, &key)
            .await
            .map_err(|e| BridgeError::Proxy(format!("load tls cert/key: {e}")));
    }

    let dir = state.storage.config_dir();
    let cert_path = dir.join("tls-cert.pem");
    let key_path = dir.join("tls-key.pem");
    if !cert_path.exists() || !key_path.exists() {
        let sans = vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "::1".to_string(),
            // A stable, non-resolving placeholder hostname so users can map it
            // (via hosts file / local DNS) to wherever the bridge runs and get
            // a name that matches the cert's SAN — e.g. on a router:
            //   192.168.2.1  local.miclawbridge.com
            // Trusting this self-signed cert then yields a warning-free HTTPS
            // origin without per-IP cert regeneration. Documented in README.
            "local.miclawbridge.com".to_string(),
        ];
        let generated = rcgen::generate_simple_self_signed(sans)
            .map_err(|e| BridgeError::Proxy(e.to_string()))?;
        std::fs::write(&cert_path, generated.cert.pem())?;
        std::fs::write(&key_path, generated.key_pair.serialize_pem())?;
        tracing::info!(
            target = "server",
            "generated self-signed TLS cert at {}",
            cert_path.display()
        );
    }
    RustlsConfig::from_pem_file(&cert_path, &key_path)
        .await
        .map_err(|e| BridgeError::Proxy(format!("load self-signed cert: {e}")))
}

pub fn router(state: Arc<BridgeState>) -> Router {
    let api = Router::new()
        .route("/api/auth/status", get(api_auth_status))
        .route("/api/auth/login", post(api_login))
        .route("/api/auth/two-factor/send", post(api_send_ticket))
        .route("/api/auth/two-factor/verify", post(api_verify_ticket))
        .route("/api/auth/refresh", post(api_refresh_session))
        .route("/api/auth/logout", post(api_logout))
        .route("/api/proxy/status", get(api_proxy_status))
        .route("/api/settings/port", post(api_set_port))
        .route("/api/models", get(api_models))
        .route("/api/logs", get(api_logs))
        .route(
            "/api/logs/verbose",
            get(api_logs_verbose_get).post(api_logs_verbose_set),
        )
        .route("/api/logs/stream", get(api_logs_stream))
        // admin auth (control-plane password)
        .route("/api/admin/session", get(api_admin_session))
        .route("/api/admin/setup", post(api_admin_setup))
        .route("/api/admin/login", post(api_admin_login))
        .route("/api/admin/logout", post(api_admin_logout))
        .route("/api/admin/password", post(api_admin_password))
        // api keys for /v1
        .route("/api/keys", get(api_keys_list).post(api_keys_create))
        .route("/api/keys/:id", axum::routing::delete(api_keys_delete))
        .route(
            "/api/settings/api-key-required",
            get(api_key_required_get).post(api_key_required_set),
        )
        .route("/api/usage", get(api_usage))
        // everything under /api requires a valid admin session once configured
        // (the guard whitelists session/setup/login itself)
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            admin_guard,
        ))
        .with_state(state.clone());

    let proxy = Router::new()
        .route("/v1/models", get(crate::proxy::openai::models))
        .route("/v1/chat/completions", post(crate::proxy::openai::chat))
        .route("/v1/responses", post(crate::proxy::openai::responses))
        .route("/v1/messages", post(crate::proxy::anthropic::messages))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            api_key_guard,
        ))
        .with_state(state.proxy.clone());

    Router::new()
        .merge(api)
        .merge(proxy)
        .fallback(static_asset)
        .layer(CorsLayer::permissive())
}

/// Endpoints reachable without an admin session (so the login UI can work).
fn admin_open_path(path: &str) -> bool {
    matches!(
        path,
        "/api/admin/session" | "/api/admin/setup" | "/api/admin/login"
    )
}

fn session_cookie(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    raw.split(';').find_map(|kv| {
        let kv = kv.trim();
        let (k, v) = kv.split_once('=')?;
        if k == "mb_session" {
            Some(v.to_string())
        } else {
            None
        }
    })
}

/// Guard `/api/*`: once an admin password is configured, every endpoint except
/// the auth ones requires a valid `mb_session` cookie.
async fn admin_guard(
    State(state): State<Arc<BridgeState>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    // Admin password authentication is DISABLED — the WebUI works without
    // a login. If you want to re-enable it, uncomment the block below and
    // remove this early return.
    return next.run(req).await;

    // --- original auth guard (keep as reference) ---
    // let path = req.uri().path();
    // if admin_open_path(path) || !state.security.is_configured() {
    //     return next.run(req).await;
    // }
    // if let Some(token) = session_cookie(req.headers()) {
    //     if state.security.validate_session(&token) {
    //         return next.run(req).await;
    //     }
    // }
    // (
    //     StatusCode::UNAUTHORIZED,
    //     Json(json!({"error": {"message": "admin authentication required"}})),
    // )
    //     .into_response()
}

#[derive(serde::Deserialize)]
struct PasswordReq {
    password: String,
}

#[derive(serde::Deserialize)]
struct ChangePasswordReq {
    old_password: String,
    new_password: String,
}

async fn api_admin_session(State(state): State<Arc<BridgeState>>, headers: HeaderMap) -> Response {
    let configured = state.security.is_configured();
    let authenticated = !configured
        || session_cookie(&headers)
            .map(|t| state.security.validate_session(&t))
            .unwrap_or(false);
    Json(json!({ "configured": configured, "authenticated": authenticated })).into_response()
}

async fn api_admin_setup(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<PasswordReq>,
) -> Response {
    match state.security.setup(&req.password) {
        Ok(()) => match state.security.login(&req.password) {
            Ok(token) => login_response(&state, token),
            Err(e) => error_response(e),
        },
        Err(e) => error_response(e),
    }
}

async fn api_admin_login(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<PasswordReq>,
) -> Response {
    match state.security.login(&req.password) {
        Ok(token) => login_response(&state, token),
        Err(e) => error_response(e),
    }
}

async fn api_admin_logout(State(state): State<Arc<BridgeState>>, headers: HeaderMap) -> Response {
    if let Some(token) = session_cookie(&headers) {
        state.security.revoke_session(&token);
    }
    let mut resp = Json(json!({"ok": true})).into_response();
    let cleared = "mb_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0";
    if let Ok(v) = HeaderValue::from_str(cleared) {
        resp.headers_mut().insert(header::SET_COOKIE, v);
    }
    resp
}

async fn api_admin_password(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<ChangePasswordReq>,
) -> Response {
    match state
        .security
        .change_password(&req.old_password, &req.new_password)
    {
        Ok(()) => match state.security.login(&req.new_password) {
            Ok(token) => login_response(&state, token),
            Err(e) => error_response(e),
        },
        Err(e) => error_response(e),
    }
}

/// Build a JSON `{ok:true}` response that also sets the session cookie.
fn login_response(state: &Arc<BridgeState>, token: String) -> Response {
    let secure = if state.storage.settings().tls_enabled {
        "; Secure"
    } else {
        ""
    };
    let cookie =
        format!("mb_session={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age=604800{secure}");
    let mut resp = Json(json!({"ok": true})).into_response();
    if let Ok(v) = HeaderValue::from_str(&cookie) {
        resp.headers_mut().insert(header::SET_COOKIE, v);
    }
    resp
}

// ---- API keys ---------------------------------------------------------------

/// Enforce API keys on `/v1/*` when `api_key_required` is on. When off, any
/// (or no) key works, preserving the original drop-in behavior.
async fn api_key_guard(
    State(state): State<Arc<BridgeState>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    if !state.storage.settings().api_key_required {
        return next.run(req).await;
    }
    let ok = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.strip_prefix("Bearer ")
                .or_else(|| s.strip_prefix("bearer "))
        })
        .map(|key| state.security.verify_key(key.trim()))
        .unwrap_or(false);
    if ok {
        next.run(req).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": {
                    "type": "invalid_request_error",
                    "message": "invalid or missing API key",
                }
            })),
        )
            .into_response()
    }
}

async fn api_keys_list(State(state): State<Arc<BridgeState>>) -> Response {
    Json(state.security.list_keys()).into_response()
}

#[derive(serde::Deserialize)]
struct CreateKeyReq {
    #[serde(default)]
    name: String,
}

async fn api_keys_create(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<CreateKeyReq>,
) -> Response {
    match state.security.create_key(&req.name) {
        Ok((view, secret)) => Json(json!({ "key": view, "secret": secret })).into_response(),
        Err(e) => error_response(e),
    }
}

async fn api_keys_delete(
    State(state): State<Arc<BridgeState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    match state.security.revoke_key(&id) {
        Ok(()) => Json(json!({"ok": true})).into_response(),
        Err(e) => error_response(e),
    }
}

async fn api_key_required_get(State(state): State<Arc<BridgeState>>) -> Response {
    Json(json!({ "required": state.storage.settings().api_key_required })).into_response()
}

#[derive(serde::Deserialize)]
struct ApiKeyRequiredReq {
    required: bool,
}

async fn api_key_required_set(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<ApiKeyRequiredReq>,
) -> Response {
    match state
        .storage
        .update_settings(|s| s.api_key_required = req.required)
    {
        Ok(_) => Json(json!({ "required": req.required })).into_response(),
        Err(e) => error_response(e),
    }
}

#[derive(serde::Deserialize)]
struct UsageQuery {
    #[serde(default)]
    window: Option<String>,
}

async fn api_usage(
    State(state): State<Arc<BridgeState>>,
    axum::extract::Query(q): axum::extract::Query<UsageQuery>,
) -> Response {
    let window = q.window.as_deref().unwrap_or("1d");
    Json(state.usage.query(window)).into_response()
}

async fn api_auth_status(State(state): State<Arc<BridgeState>>) -> Response {
    json_result(crate::service::auth_status(&state).await)
}

use serde::Deserialize;

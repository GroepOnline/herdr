use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::Json;
use axum::routing::get;
use axum::Router;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Clone)]
struct GatewayState {
    socket_path: PathBuf,
    version: String,
}

type SharedState = Arc<RwLock<GatewayState>>;

async fn socket_request(socket_path: &PathBuf, req: &Value) -> Result<Value, String> {
    let mut stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("socket connect: {}", e))?;

    let mut req_bytes = serde_json::to_vec(req).map_err(|e| e.to_string())?;
    req_bytes.push(b'\n');
    stream.write_all(&req_bytes).await.map_err(|e| e.to_string())?;

    let mut buf = vec![0u8; 65536];
    let n = stream.read(&mut buf).await.map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("empty response".to_string());
    }

    serde_json::from_slice(&buf[..n]).map_err(|e| format!("json: {}", e))
}

#[allow(dead_code)]
static REQUEST_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn next_id() -> String {
    let id = REQUEST_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("gw:{}", id)
}

// ── Handlers ──────────────────────────────────────────────────────

async fn health(State(state): State<SharedState>) -> Json<Value> {
    let s = state.read().await;
    let uptime = std::time::Instant::now()
        .elapsed()
        .as_secs();
    Json(json!({
        "status": "ok",
        "gateway_version": s.version,
        "socket": s.socket_path.display().to_string(),
        "uptime": uptime,
    }))
}

async fn get_workspaces(State(state): State<SharedState>) -> Json<Value> {
    let s = state.read().await;
    let req = json!({"id": next_id(), "method": "workspace.list", "params": {}});
    let data = socket_request(&s.socket_path, &req).await;
    Json(data.unwrap_or_else(|e| json!({"error": e})))
}

async fn get_agents(State(state): State<SharedState>) -> Json<Value> {
    let s = state.read().await;
    let req = json!({"id": next_id(), "method": "agent.list", "params": {}});
    let data = socket_request(&s.socket_path, &req).await;
    Json(data.unwrap_or_else(|e| json!({"error": e})))
}

async fn get_session(State(state): State<SharedState>) -> Json<Value> {
    let s = state.read().await;
    let req = json!({"id": next_id(), "method": "session.snapshot", "params": {}});
    let data = socket_request(&s.socket_path, &req).await;
    Json(data.unwrap_or_else(|e| json!({"error": e})))
}

async fn sse_events(State(state): State<SharedState>) -> Sse<impl tokio_stream::Stream<Item = Result<SseEvent, std::convert::Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<SseEvent, std::convert::Infallible>>();
    let socket_path = state.read().await.socket_path.clone();

    let _ = tx.send(Ok(SseEvent::default()
        .event("connected")
        .data(json!({"status": "connected"}).to_string())));

    tokio::task::spawn(async move {
        let mut stream = match UnixStream::connect(&socket_path).await {
            Ok(s) => s,
            Err(e) => {
                let _ = tx.send(Ok(SseEvent::default()
                    .event("error")
                    .data(format!("socket: {}", e))));
                return;
            }
        };

        let sub_req = json!({"id": "gw:sse", "method": "events.subscribe", "params": {"subscriptions": []}});
        let mut req_bytes = serde_json::to_vec(&sub_req).unwrap_or_default();
        req_bytes.push(b'\n');
        if stream.write_all(&req_bytes).await.is_err() {
            return;
        }

        let mut buf = vec![0u8; 32768];
        let mut line_buf = Vec::new();
        loop {
            match stream.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    for &byte in &buf[..n] {
                        if byte == b'\n' {
                            if !line_buf.is_empty() {
                                if let Ok(val) = serde_json::from_slice::<Value>(&line_buf) {
                                    let event_type = val.get("event")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("message");
                                    let id = val.get("seq")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0);
                                    let data = serde_json::to_string(&val)
                                        .unwrap_or_else(|_| "{}".to_string());
                                    let sse = SseEvent::default()
                                        .event(event_type)
                                        .id(id.to_string())
                                        .data(data);
                                    if tx.send(Ok(sse)).is_err() {
                                        return;
                                    }
                                }
                                line_buf.clear();
                            }
                        } else {
                            line_buf.push(byte);
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    let stream = UnboundedReceiverStream::new(rx);
    Sse::new(stream).keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(15)))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port: u16 = std::env::var("HERDR_GATEWAY_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7777);

    let socket_path = std::env::var("HERDR_SOCKET_PATH")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".config/herdr/herdr.sock")
        });

    let state = Arc::new(RwLock::new(GatewayState {
        socket_path: socket_path.clone(),
        version: "0.7.3-chef".to_string(),
    }));

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/session", get(get_session))
        .route("/v1/workspaces", get(get_workspaces))
        .route("/v1/agents", get(get_agents))
        .route("/v1/events", get(sse_events))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("herdr-gateway listening on http://127.0.0.1:{}", port);
    eprintln!("socket: {}", socket_path.display());
    axum::serve(listener, app).await?;

    Ok(())
}


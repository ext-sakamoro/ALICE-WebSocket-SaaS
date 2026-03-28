#![allow(dead_code)]
use axum::{extract::State, response::Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

// ── State ───────────────────────────────────────────────────
struct AppState {
    start_time: Instant,
    stats: Mutex<Stats>,
}

struct Stats {
    total_connections: u64,
    total_broadcasts: u64,
    total_rooms: u64,
    total_config_updates: u64,
    messages_sent: u64,
}

// ── Types ───────────────────────────────────────────────────
#[derive(Serialize)]
struct Health { status: String, version: String, uptime_secs: u64, total_ops: u64 }

// Connections
#[derive(Serialize)]
struct ConnectionInfo {
    conn_id: String, remote_addr: String, room: Option<String>,
    protocol: String, connected_secs: u64,
}
#[derive(Serialize)]
struct ConnectionsResponse { connections: Vec<ConnectionInfo>, total: usize }

// Broadcast
#[derive(Deserialize)]
#[allow(dead_code)]
struct BroadcastRequest {
    room: Option<String>,
    message: Option<String>,
    msg_type: Option<String>,
    exclude: Option<Vec<String>>,
}
#[derive(Serialize)]
struct BroadcastResponse {
    broadcast_id: String, status: String,
    room: Option<String>, recipients: u32,
    msg_type: String, elapsed_us: u128,
}

// Rooms
#[derive(Deserialize)]
#[allow(dead_code)]
struct RoomRequest {
    name: Option<String>,
    max_connections: Option<u32>,
    ttl_secs: Option<u64>,
    persistent: Option<bool>,
}
#[derive(Serialize)]
struct RoomResponse {
    room_id: String, status: String, name: String,
    max_connections: u32, ttl_secs: u64, persistent: bool,
    current_connections: u32, elapsed_us: u128,
}

// Config
#[derive(Deserialize)]
#[allow(dead_code)]
struct ConfigRequest {
    max_connections: Option<u32>,
    ping_interval_secs: Option<u64>,
    pong_timeout_secs: Option<u64>,
    max_message_size_kb: Option<u32>,
}
#[derive(Serialize)]
struct ConfigResponse {
    status: String, max_connections: u32, ping_interval_secs: u64,
    pong_timeout_secs: u64, max_message_size_kb: u32, elapsed_us: u128,
}

// Stats
#[derive(Serialize)]
struct StatsResponse {
    total_connections: u64, total_broadcasts: u64, total_rooms: u64,
    total_config_updates: u64, messages_sent: u64,
}

// ── Main ────────────────────────────────────────────────────
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "websocket_engine=info".into()))
        .init();
    let state = Arc::new(AppState {
        start_time: Instant::now(),
        stats: Mutex::new(Stats {
            total_connections: 0, total_broadcasts: 0, total_rooms: 0,
            total_config_updates: 0, messages_sent: 0,
        }),
    });
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/ws/connections", get(connections))
        .route("/api/v1/ws/broadcast", post(broadcast))
        .route("/api/v1/ws/rooms", post(rooms))
        .route("/api/v1/ws/config", post(config))
        .route("/api/v1/ws/stats", get(stats))
        .layer(cors).layer(TraceLayer::new_for_http()).with_state(state);
    let addr = std::env::var("WS_ADDR").unwrap_or_else(|_| "0.0.0.0:8131".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("WebSocket Engine on {addr}");
    axum::serve(listener, app).await.unwrap();
}

// ── Handlers ────────────────────────────────────────────────
async fn health(State(s): State<Arc<AppState>>) -> Json<Health> {
    let st = s.stats.lock().unwrap();
    Json(Health {
        status: "ok".into(), version: env!("CARGO_PKG_VERSION").into(),
        uptime_secs: s.start_time.elapsed().as_secs(),
        total_ops: st.total_connections + st.total_broadcasts + st.total_rooms,
    })
}

async fn connections(State(s): State<Arc<AppState>>) -> Json<ConnectionsResponse> {
    s.stats.lock().unwrap().total_connections += 1;
    let sample = vec![
        ConnectionInfo {
            conn_id: uuid::Uuid::new_v4().to_string(),
            remote_addr: "203.0.113.10:54321".into(),
            room: Some("chat-room-1".into()),
            protocol: "ws".into(), connected_secs: 142,
        },
        ConnectionInfo {
            conn_id: uuid::Uuid::new_v4().to_string(),
            remote_addr: "203.0.113.20:54322".into(),
            room: None,
            protocol: "wss".into(), connected_secs: 37,
        },
    ];
    let total = sample.len();
    Json(ConnectionsResponse { connections: sample, total })
}

async fn broadcast(State(s): State<Arc<AppState>>, Json(req): Json<BroadcastRequest>) -> Json<BroadcastResponse> {
    let t = Instant::now();
    let room = req.room;
    let msg_type = req.msg_type.unwrap_or_else(|| "text".into());
    let recipients = if room.is_some() { 42u32 } else { 1024u32 };
    {
        let mut st = s.stats.lock().unwrap();
        st.total_broadcasts += 1;
        st.messages_sent += recipients as u64;
    }
    Json(BroadcastResponse {
        broadcast_id: uuid::Uuid::new_v4().to_string(),
        status: "sent".into(), room, recipients, msg_type,
        elapsed_us: t.elapsed().as_micros(),
    })
}

async fn rooms(State(s): State<Arc<AppState>>, Json(req): Json<RoomRequest>) -> Json<RoomResponse> {
    let t = Instant::now();
    let name = req.name.unwrap_or_else(|| "default-room".into());
    let max_connections = req.max_connections.unwrap_or(100);
    let ttl_secs = req.ttl_secs.unwrap_or(3600);
    let persistent = req.persistent.unwrap_or(false);
    s.stats.lock().unwrap().total_rooms += 1;
    Json(RoomResponse {
        room_id: uuid::Uuid::new_v4().to_string(),
        status: "created".into(), name, max_connections, ttl_secs,
        persistent, current_connections: 0,
        elapsed_us: t.elapsed().as_micros(),
    })
}

async fn config(State(s): State<Arc<AppState>>, Json(req): Json<ConfigRequest>) -> Json<ConfigResponse> {
    let t = Instant::now();
    let max_connections = req.max_connections.unwrap_or(10_000);
    let ping_interval_secs = req.ping_interval_secs.unwrap_or(30);
    let pong_timeout_secs = req.pong_timeout_secs.unwrap_or(10);
    let max_message_size_kb = req.max_message_size_kb.unwrap_or(64);
    s.stats.lock().unwrap().total_config_updates += 1;
    Json(ConfigResponse {
        status: "applied".into(), max_connections, ping_interval_secs,
        pong_timeout_secs, max_message_size_kb,
        elapsed_us: t.elapsed().as_micros(),
    })
}

async fn stats(State(s): State<Arc<AppState>>) -> Json<StatsResponse> {
    let st = s.stats.lock().unwrap();
    Json(StatsResponse {
        total_connections: st.total_connections,
        total_broadcasts: st.total_broadcasts,
        total_rooms: st.total_rooms,
        total_config_updates: st.total_config_updates,
        messages_sent: st.messages_sent,
    })
}

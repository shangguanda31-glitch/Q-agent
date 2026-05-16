use std::sync::Arc;
use axum::{Router, Json, extract::{State, Path}, response::{sse::{Event, Sse}, IntoResponse, Html}, routing::{get, post}};
use futures_util::stream::Stream;
use std::convert::Infallible;
use tokio::sync::broadcast;
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::store::{EventStore, ScheduleStore, ScheduleEntry, NoteStore, MemoryStore, NoteEntry, MemoryEntry, ExclusionStore, ExclusionEntry};
use crate::napcat::types::ProcessedEvent;
use crate::napcat::api::NapCatApi;

#[derive(Clone)]
pub struct AppState {
    pub event_tx: Arc<broadcast::Sender<ProcessedEvent>>,
    pub store: Arc<EventStore>,
    pub sched: Arc<ScheduleStore>,
    pub notes: Arc<NoteStore>,
    pub memories: Arc<MemoryStore>,
    pub exclusions: Arc<ExclusionStore>,
    pub napcat: Arc<NapCatApi>,
}

pub fn router(
    event_tx: Arc<broadcast::Sender<ProcessedEvent>>,
    store: Arc<EventStore>,
    sched: Arc<ScheduleStore>,
    notes: Arc<NoteStore>,
    memories: Arc<MemoryStore>,
    exclusions: Arc<ExclusionStore>,
    napcat: Arc<NapCatApi>,
) -> Router {
    let state = AppState { event_tx, store, sched, notes, memories, exclusions, napcat };
    Router::new()
        .route("/", get(index))
        .route("/events", get(sse_handler))
        .route("/api/history", get(history_handler))
        .route("/api/schedules", get(schedule_list))
        .route("/api/schedules/done", post(schedule_done))
        .route("/api/schedules/delete", post(schedule_delete))
        .route("/api/notes", get(notes_list))
        .route("/api/memories", get(memories_list))
        .route("/api/workspace", get(workspace_list))
        .route("/api/workspace/{*path}", get(workspace_file))
        .route("/api/claude-progress", get(claude_progress))
        .nest_service("/images", ServeDir::new("image_cache"))
        .route("/api/memories/delete", post(memory_delete))
        .route("/api/notes/delete", post(note_delete))
        .nest_service("/workspace_files", ServeDir::new("claude_workspace"))
        // Exclusion management
        .route("/api/exclusions", get(exclusion_list))
        .route("/api/exclusions/add", post(exclusion_add))
        .route("/api/exclusions/remove", post(exclusion_remove))
        // Chat sources
        .route("/api/chat-sources", get(chat_sources))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn index() -> impl IntoResponse { Html(include_str!("static/index.html")) }

async fn sse_handler(State(state): State<AppState>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.event_tx.subscribe();
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(event) => { let json = serde_json::to_string(&event).unwrap_or_default(); yield Ok(Event::default().data(json)); }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new().interval(std::time::Duration::from_secs(30)).text("keep-alive"))
}

async fn history_handler(State(state): State<AppState>) -> Json<Vec<ProcessedEvent>> { Json(state.store.recent()) }
async fn schedule_list(State(state): State<AppState>) -> Json<Vec<ScheduleEntry>> { Json(state.sched.list()) }
async fn notes_list(State(state): State<AppState>) -> Json<Vec<NoteEntry>> { Json(state.notes.list()) }
async fn memories_list(State(state): State<AppState>) -> Json<Vec<MemoryEntry>> { Json(state.memories.all()) }

#[derive(serde::Deserialize)]
struct IdReq { id: String }
async fn schedule_done(State(state): State<AppState>, Json(req): Json<IdReq>) -> Json<serde_json::Value> { Json(serde_json::json!({"ok": state.sched.mark_done(&req.id)})) }
async fn schedule_delete(State(state): State<AppState>, Json(req): Json<IdReq>) -> Json<serde_json::Value> { Json(serde_json::json!({"ok": state.sched.delete(&req.id)})) }

async fn workspace_list() -> Json<Vec<String>> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir("claude_workspace") {
        for e in entries.flatten() {
            if let Ok(name) = e.file_name().into_string() { files.push(name); }
        }
    }
    Json(files)
}

async fn workspace_file(Path(path): Path<String>) -> impl IntoResponse {
    let path = format!("claude_workspace/{}", path);
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => Html(format!("<pre>{}</pre>", html_escape(&content))),
        Err(_) => Html("<p>File not found or binary</p>".to_string()),
    }
}

async fn claude_progress() -> Json<serde_json::Value> {
    match tokio::fs::read_to_string("claude_workspace/.claude_progress").await {
        Ok(content) => Json(serde_json::json!({"progress": content})),
        Err(_) => Json(serde_json::json!({"progress": ""})),
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

async fn memory_delete(State(state): State<AppState>, Json(req): Json<IdReq>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": state.memories.delete(&req.id)}))
}
async fn note_delete(State(state): State<AppState>, Json(req): Json<IdReq>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": state.notes.delete(&req.id)}))
}

// === Exclusion handlers ===

async fn exclusion_list(State(state): State<AppState>) -> Json<Vec<ExclusionEntry>> {
    Json(state.exclusions.list())
}

#[derive(serde::Deserialize)]
struct ExclusionReq {
    exclude_type: String, // "group" or "user"
    target_id: i64,
    note: Option<String>,
}

async fn exclusion_add(State(state): State<AppState>, Json(req): Json<ExclusionReq>) -> Json<serde_json::Value> {
    let ok = state.exclusions.add(&req.exclude_type, req.target_id, req.note.as_deref().unwrap_or(""));
    Json(serde_json::json!({"ok": ok}))
}

async fn exclusion_remove(State(state): State<AppState>, Json(req): Json<ExclusionReq>) -> Json<serde_json::Value> {
    let ok = state.exclusions.remove(&req.exclude_type, req.target_id);
    Json(serde_json::json!({"ok": ok}))
}

// === Chat sources (groups + friends from NapCat) ===

async fn chat_sources(State(state): State<AppState>) -> Json<serde_json::Value> {
    let groups = state.napcat.get_group_list().await;
    let friends = state.napcat.get_friend_list().await;
    let excluded: std::collections::HashSet<(String, i64)> = state.exclusions.list().into_iter()
        .map(|e| (e.exclude_type, e.target_id)).collect();

    let group_list: Vec<serde_json::Value> = groups.iter().map(|g| {
        let id = g.get("group_id").and_then(|v| v.as_i64()).unwrap_or(0);
        let name = g.get("group_name").and_then(|v| v.as_str()).unwrap_or("未知群").to_string();
        let excluded = excluded.contains(&("group".to_string(), id));
        serde_json::json!({"id": id, "name": name, "type": "group", "excluded": excluded})
    }).collect();

    let friend_list: Vec<serde_json::Value> = friends.iter().map(|f| {
        let id = f.get("user_id").and_then(|v| v.as_i64()).unwrap_or(0);
        let name = f.get("nickname").or_else(|| f.get("remark")).and_then(|v| v.as_str()).unwrap_or("未知好友").to_string();
        let excluded = excluded.contains(&("user".to_string(), id));
        serde_json::json!({"id": id, "name": name, "type": "user", "excluded": excluded})
    }).collect();

    Json(serde_json::json!({"groups": group_list, "friends": friend_list}))
}

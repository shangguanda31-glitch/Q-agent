use std::sync::Arc;
use axum::{Router, Json, extract::{State, Path}, response::{sse::{Event, Sse}, IntoResponse, Html}, routing::{get, post}};
use futures_util::stream::Stream;
use std::convert::Infallible;
use tokio::sync::broadcast;
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::store::{EventStore, ScheduleStore, ScheduleEntry, NoteStore, MemoryStore, NoteEntry, MemoryEntry};
use crate::napcat::types::ProcessedEvent;

#[derive(Clone)]
pub struct AppState {
    pub event_tx: Arc<broadcast::Sender<ProcessedEvent>>,
    pub store: Arc<EventStore>,
    pub sched: Arc<ScheduleStore>,
    pub notes: Arc<NoteStore>,
    pub memories: Arc<MemoryStore>,
}

pub fn router(
    event_tx: Arc<broadcast::Sender<ProcessedEvent>>,
    store: Arc<EventStore>,
    sched: Arc<ScheduleStore>,
    notes: Arc<NoteStore>,
    memories: Arc<MemoryStore>,
) -> Router {
    let state = AppState { event_tx, store, sched, notes, memories };
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
        .nest_service("/images", ServeDir::new("image_cache"))
        .nest_service("/workspace_files", ServeDir::new("claude_workspace"))
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

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

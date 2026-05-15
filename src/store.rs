use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::napcat::types::ProcessedEvent;

const MAX_EVENTS: usize = 500;

// === Event Store (in-memory message history) ===

pub struct EventStore {
    events: Mutex<VecDeque<ProcessedEvent>>,
    file_path: PathBuf,
}

impl EventStore {
    pub fn new(data_dir: &str) -> Self {
        let dir = PathBuf::from(data_dir);
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("message_history.json");

        let events = if file_path.exists() {
            std::fs::read_to_string(&file_path)
                .ok()
                .and_then(|s| serde_json::from_str::<VecDeque<ProcessedEvent>>(&s).ok())
                .unwrap_or_else(|| VecDeque::with_capacity(MAX_EVENTS))
        } else {
            VecDeque::with_capacity(MAX_EVENTS)
        };

        Self { events: Mutex::new(events), file_path }
    }

    fn save(&self) {
        if let Ok(events) = self.events.lock() {
            if let Ok(json) = serde_json::to_string(&*events) {
                let _ = std::fs::write(&self.file_path, &json);
            }
        }
    }

    pub fn push(&self, event: ProcessedEvent) {
        let mut events = self.events.lock().unwrap();
        if events.len() >= MAX_EVENTS {
            events.pop_front();
        }
        events.push_back(event);
        drop(events);
        self.save();
    }

    pub fn recent(&self) -> Vec<ProcessedEvent> {
        let events = self.events.lock().unwrap();
        events.iter().rev().take(100).cloned().collect()
    }
}

// === Schedule Store ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub id: String,
    pub title: String,
    pub time: Option<String>,
    pub time_parsed: Option<String>,   // ISO 8601 if parsable
    pub description: Option<String>,
    pub source: String,
    pub source_user: String,
    pub status: String,         // pending | confirmed | done | reminded
    pub created_at: String,
}

pub struct ScheduleStore {
    entries: Mutex<Vec<ScheduleEntry>>,
    file_path: PathBuf,
}

impl ScheduleStore {
    pub fn new(data_dir: &str) -> Self {
        let dir = PathBuf::from(data_dir);
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("schedules.json");

        let entries = if file_path.exists() {
            std::fs::read_to_string(&file_path)
                .ok()
                .and_then(|s| serde_json::from_str::<Vec<ScheduleEntry>>(&s).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self { entries: Mutex::new(entries), file_path }
    }

    fn save(&self) {
        if let Ok(entries) = self.entries.lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*entries) {
                let _ = std::fs::write(&self.file_path, &json);
            }
        }
    }

    pub fn list(&self) -> Vec<ScheduleEntry> {
        let entries = self.entries.lock().unwrap();
        let mut sorted = entries.clone();
        sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        sorted
    }

    pub fn create(&self, title: String, time: Option<String>, description: Option<String>,
                  source: String, source_user: String) -> ScheduleEntry {
        let time_parsed = time.as_ref().and_then(|t| try_parse_time(t));
        let entry = ScheduleEntry {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            time,
            time_parsed,
            description,
            source,
            source_user,
            status: "pending".to_string(),
            created_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string(),
        };
        let mut entries = self.entries.lock().unwrap();
        entries.push(entry.clone());
        drop(entries);
        self.save();
        entry
    }

    /// Get schedules due within `minutes` that haven't been reminded yet
    pub fn get_due_for_reminder(&self, within_minutes: i64) -> Vec<ScheduleEntry> {
        let entries = self.entries.lock().unwrap();
        let now = chrono::Local::now();
        entries.iter().filter(|e| {
            if e.status != "pending" && e.status != "confirmed" {
                return false;
            }
            if let Some(ref tp) = e.time_parsed {
                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(tp, "%Y-%m-%d %H:%M") {
                    let dt_local: chrono::DateTime<chrono::Local> = chrono::TimeZone::from_local_datetime(&chrono::Local, &dt).unwrap();
                    let mins_until = (dt_local - now).num_minutes();
                    mins_until >= 0 && mins_until <= within_minutes
                } else { false }
            } else { false }
        }).cloned().collect()
    }

    pub fn mark_reminded(&self, id: &str) -> bool {
        let mut entries = self.entries.lock().unwrap();
        if let Some(e) = entries.iter_mut().find(|e| e.id == id) {
            e.status = "reminded".to_string();
            drop(entries);
            self.save();
            true
        } else { false }
    }

    pub fn mark_done(&self, id: &str) -> bool {
        let mut entries = self.entries.lock().unwrap();
        if let Some(e) = entries.iter_mut().find(|e| e.id == id) {
            e.status = "done".to_string();
            drop(entries);
            self.save();
            true
        } else {
            false
        }
    }

    pub fn delete(&self, id: &str) -> bool {
        let mut entries = self.entries.lock().unwrap();
        let len_before = entries.len();
        entries.retain(|e| e.id != id);
        let removed = entries.len() < len_before;
        drop(entries);
        if removed { self.save(); }
        removed
    }
}

// === Memory Store (persistent knowledge) ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub source: String,
    pub created_at: String,
}

pub struct MemoryStore {
    entries: Mutex<Vec<MemoryEntry>>,
    file_path: PathBuf,
}

impl MemoryStore {
    pub fn new(data_dir: &str) -> Self {
        let dir = PathBuf::from(data_dir);
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("memories.json");
        let entries = if file_path.exists() {
            std::fs::read_to_string(&file_path)
                .ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
        } else { Vec::new() };
        Self { entries: Mutex::new(entries), file_path }
    }

    fn save(&self) {
        if let Ok(entries) = self.entries.lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*entries) {
                let _ = std::fs::write(&self.file_path, &json);
            }
        }
    }

    pub fn write(&self, content: String, tags: Vec<String>, source: String) -> MemoryEntry {
        let entry = MemoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            tags,
            source,
            created_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string(),
        };
        let mut entries = self.entries.lock().unwrap();
        entries.push(entry.clone());
        drop(entries);
        self.save();
        entry
    }

    pub fn read(&self, query: &str, limit: usize) -> Vec<MemoryEntry> {
        let entries = self.entries.lock().unwrap();
        let q = query.to_lowercase();
        let mut matched: Vec<_> = entries.iter().filter(|e| {
            e.content.to_lowercase().contains(&q)
                || e.tags.iter().any(|t| t.to_lowercase().contains(&q))
        }).cloned().collect();
        matched.reverse();
        matched.truncate(limit);
        matched
    }

    pub fn load_context(&self, query: &str, max_entries: usize) -> String {
        let relevant = self.read(query, max_entries);
        if relevant.is_empty() { return String::new(); }
        let mut lines = vec!["[相关记忆]".to_string()];
        for m in &relevant {
            lines.push(format!("- {} (标签: {})", m.content, m.tags.join(", ")));
        }
        lines.join("\n")
    }

    pub fn all(&self) -> Vec<MemoryEntry> {
        let entries = self.entries.lock().unwrap();
        let mut sorted = entries.clone();
        sorted.reverse();
        sorted
    }
}

// === Note Store (conversation notes) ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteEntry {
    pub id: String,
    pub content: String,
    pub speaker: String,
    pub speaker_id: i64,
    pub source: String,          // "private" | "group"
    pub group_id: Option<i64>,
    pub message_time: String,
    pub created_at: String,
}

pub struct NoteStore {
    entries: Mutex<Vec<NoteEntry>>,
    file_path: PathBuf,
}

impl NoteStore {
    pub fn new(data_dir: &str) -> Self {
        let dir = PathBuf::from(data_dir);
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("notes.json");
        let entries = if file_path.exists() {
            std::fs::read_to_string(&file_path)
                .ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
        } else { Vec::new() };
        Self { entries: Mutex::new(entries), file_path }
    }

    fn save(&self) {
        if let Ok(entries) = self.entries.lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*entries) {
                let _ = std::fs::write(&self.file_path, &json);
            }
        }
    }

    pub fn create(&self, content: String, speaker: String, speaker_id: i64,
                  source: String, group_id: Option<i64>, message_time: String) -> NoteEntry {
        let entry = NoteEntry {
            id: uuid::Uuid::new_v4().to_string(),
            content, speaker, speaker_id, source, group_id, message_time,
            created_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string(),
        };
        let mut entries = self.entries.lock().unwrap();
        entries.push(entry.clone());
        drop(entries);
        self.save();
        entry
    }

    pub fn list(&self) -> Vec<NoteEntry> {
        let entries = self.entries.lock().unwrap();
        let mut sorted = entries.clone();
        sorted.reverse();
        sorted
    }
}

// === Message History Store (per-chat conversation history for agent) ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,  // "user" | "assistant" | "tool"
    pub content: String,
    pub name: Option<String>,
}

pub struct MessageHistoryStore {
    history: Mutex<HashMap<String, VecDeque<ChatMessage>>>,
    max_per_chat: usize,
}

impl MessageHistoryStore {
    pub fn new(max_per_chat: usize) -> Self {
        Self {
            history: Mutex::new(HashMap::new()),
            max_per_chat,
        }
    }

    pub fn push(&self, chat_id: &str, msg: ChatMessage) {
        let mut history = self.history.lock().unwrap();
        let entries = history.entry(chat_id.to_string()).or_insert_with(|| VecDeque::with_capacity(self.max_per_chat));
        if entries.len() >= self.max_per_chat {
            entries.pop_front();
        }
        entries.push_back(msg);
    }

    pub fn recent(&self, chat_id: &str, count: usize) -> Vec<ChatMessage> {
        let history = self.history.lock().unwrap();
        if let Some(entries) = history.get(chat_id) {
            entries.iter().rev().take(count).cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn clear(&self, chat_id: &str) {
        let mut history = self.history.lock().unwrap();
        history.remove(chat_id);
    }
}

/// Simple Chinese time expression parser
fn try_parse_time(s: &str) -> Option<String> {
    let now = chrono::Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    let tomorrow = (now + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();

    // "YYYY-MM-DD HH:MM" already
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s.trim(), "%Y-%m-%d %H:%M") {
        return Some(dt.format("%Y-%m-%d %H:%M").to_string());
    }

    // Extract X点
    let hour = s.find('点').and_then(|pos| {
        let before = &s[..pos];
        before.chars().rev().take(2).collect::<String>().chars().rev().collect::<String>()
            .trim().parse::<u32>().ok()
    });

    if let Some(mut h) = hour {
        if s.contains("下午") || s.contains("晚上") { if h < 12 { h += 12; } }
        let day = if s.contains("明天") { &tomorrow } else { &today };
        return Some(format!("{} {:02}:00", day, h));
    }

    None
}

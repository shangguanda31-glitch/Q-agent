use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Mutex;
use parking_lot;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::napcat::types::ProcessedEvent;

// === Database backend ===

fn open_db(path: &str) -> parking_lot::Mutex<Connection> {
    let conn = Connection::open(path).expect("DB open");
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;").ok();
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS events(id INTEGER PRIMARY KEY AUTOINCREMENT,time TEXT,message_type TEXT,group_id INTEGER,user_id INTEGER,sender_name TEXT,raw_text TEXT,has_image INTEGER,image_urls TEXT,has_file INTEGER,file_name TEXT,analysis TEXT,raw_json TEXT);
         CREATE TABLE IF NOT EXISTS schedules(id TEXT PRIMARY KEY,title TEXT,time TEXT,time_parsed TEXT,description TEXT,source TEXT,source_user TEXT,status TEXT,created_at TEXT);
         CREATE TABLE IF NOT EXISTS memories(id TEXT PRIMARY KEY,content TEXT,tags TEXT,source TEXT,created_at TEXT,embedding TEXT);
         CREATE TABLE IF NOT EXISTS notes(id TEXT PRIMARY KEY,content TEXT,speaker TEXT,speaker_id INTEGER,source TEXT,group_id INTEGER,message_time TEXT,created_at TEXT);"
    ).ok();
    parking_lot::Mutex::new(conn)
}

fn get_str(r: &rusqlite::Row, i: usize) -> String { r.get::<_, String>(i).unwrap_or_default() }

// === Event Store ===

pub struct EventStore {
    db: parking_lot::Mutex<Connection>,
    cache: Mutex<VecDeque<ProcessedEvent>>,
}

impl EventStore {
    pub fn new(data_dir: &str) -> Self {
        let dir = PathBuf::from(data_dir);
        std::fs::create_dir_all(&dir).ok();
        let db = open_db(&dir.join("data.db").to_string_lossy());
        Self::migrate_json(&dir, "message_history.json", &db);
        Self { db, cache: Mutex::new(VecDeque::with_capacity(500)) }
    }

    fn migrate_json(dir: &PathBuf, name: &str, db: &parking_lot::Mutex<Connection>) {
        let path = dir.join(name);
        if !path.exists() { return; }
        if let Ok(s) = std::fs::read_to_string(&path) {
            if let Ok(events) = serde_json::from_str::<Vec<ProcessedEvent>>(&s) {
                let conn = db.lock();
                for ev in &events {
                    conn.execute("INSERT OR IGNORE INTO events(time,message_type,group_id,user_id,sender_name,raw_text,has_image,image_urls,has_file,file_name,analysis,raw_json)VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                        params![ev.time,ev.message_type,ev.group_id,ev.user_id,ev.sender_name,ev.raw_text,ev.has_image as i32,ev.image_urls.join(","),ev.has_file as i32,ev.file_name,ev.analysis.as_ref().map(|a|serde_json::to_string(a).unwrap_or_default()),ev.raw_json]).ok();
                }
            }
        }
        let _ = std::fs::rename(&path, dir.join(format!("{}.bak", name)));
    }

    pub fn push(&self, ev: ProcessedEvent) {
        let conn = self.db.lock();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0)).unwrap_or(0);
        if count >= 500 { conn.execute("DELETE FROM events WHERE id IN (SELECT id FROM events ORDER BY id ASC LIMIT ?)", params![count - 500 + 1]).ok(); }
        conn.execute("INSERT INTO events(time,message_type,group_id,user_id,sender_name,raw_text,has_image,image_urls,has_file,file_name,analysis,raw_json)VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![ev.time,ev.message_type,ev.group_id,ev.user_id,ev.sender_name,ev.raw_text,ev.has_image as i32,ev.image_urls.join(","),ev.has_file as i32,ev.file_name,ev.analysis.as_ref().map(|a|serde_json::to_string(a).unwrap_or_default()),ev.raw_json]).ok();
        drop(conn);
        self.cache.lock().unwrap().push_back(ev);
    }

    pub fn recent(&self) -> Vec<ProcessedEvent> {
        let conn = self.db.lock();
        let mut stmt = conn.prepare("SELECT time,message_type,group_id,user_id,sender_name,raw_text,has_image,image_urls,has_file,file_name,analysis,raw_json FROM events ORDER BY id DESC LIMIT 100").unwrap();
        let mut out = Vec::new();
        if let Ok(rows) = stmt.query_map([], |r| {
            Ok(ProcessedEvent {
                time: r.get(0)?, message_type: r.get(1)?, group_id: r.get(2)?,
                user_id: r.get(3)?, sender_name: r.get(4)?, raw_text: r.get(5)?,
                has_image: r.get::<_,i32>(6).unwrap_or(0)!=0,
                image_urls: get_str(r,7).split(',').filter_map(|s|if s.is_empty(){None}else{Some(s.to_string())}).collect(),
                has_file: r.get::<_,i32>(8).unwrap_or(0)!=0, file_name: r.get(9)?, group_name: None,
                analysis: r.get::<_,Option<String>>(10).unwrap_or(None).and_then(|a|serde_json::from_str(&a).ok()),
                raw_json: r.get(11)?,
            })
        }) { for row in rows { if let Ok(e) = row { out.push(e); } } }
        out
    }
}

// === Schedule Store ===

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ScheduleEntry {
    pub id: String, pub title: String, pub time: Option<String>,
    pub time_parsed: Option<String>, pub description: Option<String>,
    pub source: String, pub source_user: String, pub status: String, pub created_at: String,
}

pub struct ScheduleStore(parking_lot::Mutex<Connection>);

impl ScheduleStore {
    pub fn new(data_dir: &str) -> Self {
        let dir = PathBuf::from(data_dir);
        let db = open_db(&dir.join("data.db").to_string_lossy());
        let path = dir.join("schedules.json");
        if path.exists() {
            if let Ok(s) = std::fs::read_to_string(&path) {
                if let Ok(entries) = serde_json::from_str::<Vec<ScheduleEntry>>(&s) {
                    let conn = db.lock();
                    for e in &entries { conn.execute("INSERT OR IGNORE INTO schedules VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![e.id,e.title,e.time,e.time_parsed,e.description,e.source,e.source_user,e.status,e.created_at]).ok(); }
                }
            }
            let _ = std::fs::rename(&path, dir.join("schedules.json.bak"));
        }
        Self(db)
    }

    pub fn list(&self) -> Vec<ScheduleEntry> {
        let conn = self.0.lock();
        let mut stmt = conn.prepare("SELECT id,title,time,time_parsed,description,source,source_user,status,created_at FROM schedules ORDER BY created_at DESC").unwrap();
        let mut out = Vec::new();
        if let Ok(rows) = stmt.query_map([], |r| Ok(ScheduleEntry{id:r.get(0)?,title:r.get(1)?,time:r.get(2)?,time_parsed:r.get(3)?,description:r.get(4)?,source:r.get(5)?,source_user:r.get(6)?,status:r.get(7)?,created_at:r.get(8)?})) {
            for row in rows { if let Ok(e) = row { out.push(e); } }
        }
        out
    }

    pub fn create(&self, title: String, time: Option<String>, description: Option<String>, source: String, source_user: String) -> ScheduleEntry {
        let entry = ScheduleEntry{id:uuid::Uuid::new_v4().to_string(),title,time:time.clone(),time_parsed:time.as_ref().and_then(|t|try_parse_time(t)),description,source,source_user,status:"pending".to_string(),created_at:chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string()};
        self.0.lock().execute("INSERT OR REPLACE INTO schedules VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![entry.id,entry.title,entry.time,entry.time_parsed,entry.description,entry.source,entry.source_user,entry.status,entry.created_at]).ok();
        entry
    }

    pub fn mark_done(&self, id: &str) -> bool { self.0.lock().execute("UPDATE schedules SET status='done' WHERE id=?1", params![id]).ok().is_some() }
    pub fn mark_reminded(&self, id: &str) -> bool { self.0.lock().execute("UPDATE schedules SET status='reminded' WHERE id=?1", params![id]).ok().is_some() }
    pub fn delete(&self, id: &str) -> bool { self.0.lock().execute("DELETE FROM schedules WHERE id=?1", params![id]).ok().is_some() }
    pub fn update_description(&self, id: &str, desc: &str) -> bool { self.0.lock().execute("UPDATE schedules SET description=?1 WHERE id=?2", params![desc, id]).ok().is_some() }
    pub fn get_due_for_reminder(&self, within_minutes: i64) -> Vec<ScheduleEntry> {
        let now = chrono::Local::now();
        self.list().into_iter().filter(|e| (e.status=="pending"||e.status=="confirmed") && e.time_parsed.as_ref().and_then(|tp|chrono::NaiveDateTime::parse_from_str(tp,"%Y-%m-%d %H:%M").ok()).map(|dt|{let dtl:chrono::DateTime<chrono::Local>=chrono::TimeZone::from_local_datetime(&chrono::Local,&dt).unwrap();(dtl-now).num_minutes()}).map(|m|m>=0&&m<=within_minutes).unwrap_or(false)).collect()
    }
}

// === Memory Store ===

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MemoryEntry {
    pub id: String, pub content: String, pub tags: Vec<String>,
    pub source: String, pub created_at: String, pub embedding: Option<Vec<f32>>,
}

pub struct MemoryStore(parking_lot::Mutex<Connection>);

impl MemoryStore {
    pub fn new(data_dir: &str) -> Self {
        let dir = PathBuf::from(data_dir);
        let db = open_db(&dir.join("data.db").to_string_lossy());
        let path = dir.join("memories.json");
        if path.exists() {
            if let Ok(s) = std::fs::read_to_string(&path) {
                if let Ok(entries) = serde_json::from_str::<Vec<MemoryEntry>>(&s) {
                    let conn = db.lock();
                    for e in &entries { conn.execute("INSERT OR IGNORE INTO memories VALUES(?1,?2,?3,?4,?5,?6)", params![e.id,e.content,e.tags.join(","),e.source,e.created_at,e.embedding.as_ref().map(|v|serde_json::to_string(v).unwrap_or_default())]).ok(); }
                }
            }
            let _ = std::fs::rename(&path, dir.join("memories.json.bak"));
        }
        Self(db)
    }

    pub fn write(&self, content: String, tags: Vec<String>, source: String, embedding: Option<Vec<f32>>) -> MemoryEntry {
        let entry = MemoryEntry{id:uuid::Uuid::new_v4().to_string(),content,tags:tags.clone(),source,created_at:chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string(),embedding};
        self.0.lock().execute("INSERT OR REPLACE INTO memories VALUES(?1,?2,?3,?4,?5,?6)", params![entry.id,entry.content,tags.join(","),entry.source,entry.created_at,entry.embedding.as_ref().map(|v|serde_json::to_string(v).unwrap_or_default())]).ok();
        entry
    }

    pub fn read(&self, query_embedding: Option<&[f32]>, query_text: &str, limit: usize) -> Vec<MemoryEntry> {
        let q = format!("%{}%", query_text);
        let mut results: Vec<MemoryEntry> = Vec::new();
        if let Some(conn) = self.0.try_lock() {
            if let Ok(mut stmt) = conn.prepare("SELECT id,content,tags,source,created_at,embedding FROM memories WHERE content LIKE ?1 ORDER BY rowid DESC LIMIT ?2") {
                if let Ok(rows) = stmt.query_map(params![q, limit as i64], |r| {
                    Ok(MemoryEntry{id:r.get(0)?,content:r.get(1)?,tags:get_str(r,2).split(',').filter_map(|s|if s.is_empty(){None}else{Some(s.to_string())}).collect(),source:r.get(3)?,created_at:r.get(4)?,embedding:r.get::<_,Option<String>>(5).unwrap_or(None).and_then(|s|serde_json::from_str(&s).ok())})
                }) {
                    for row in rows { if let Ok(e) = row { results.push(e); } }
                }
            }
        }
        if let Some(qe) = query_embedding {
            results.sort_by(|a,b|{let sa=a.embedding.as_deref().map(|e|crate::store::cosine_similarity(qe,e)).unwrap_or(0.0);let sb=b.embedding.as_deref().map(|e|crate::store::cosine_similarity(qe,e)).unwrap_or(0.0);sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)});
        }
        results.truncate(limit);
        results
    }

    pub fn all(&self) -> Vec<MemoryEntry> {
        let conn = self.0.lock();
        let mut stmt = conn.prepare("SELECT id,content,tags,source,created_at,embedding FROM memories ORDER BY rowid DESC").unwrap();
        let mut out = Vec::new();
        if let Ok(rows) = stmt.query_map([], |r| Ok(MemoryEntry{id:r.get(0)?,content:r.get(1)?,tags:get_str(r,2).split(',').filter_map(|s|if s.is_empty(){None}else{Some(s.to_string())}).collect(),source:r.get(3)?,created_at:r.get(4)?,embedding:r.get::<_,Option<String>>(5).unwrap_or(None).and_then(|s|serde_json::from_str(&s).ok())})) {
            for row in rows { if let Ok(e) = row { out.push(e); } }
        }
        out
    }

    pub fn load_context(&self, query: &str, max_entries: usize) -> String {
        let entries = self.read(None, query, max_entries);
        if entries.is_empty() { return String::new(); }
        let mut lines = vec!["[相关记忆]".to_string()];
        for m in &entries { lines.push(format!("- {} (标签: {})", m.content, m.tags.join(", "))); }
        lines.join("\n")
    }
    pub fn delete(&self, id: &str) -> bool {
        self.0.lock().execute("DELETE FROM memories WHERE id=?1", params![id]).ok().is_some()
    }
}

// === Note Store ===

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NoteEntry {
    pub id: String, pub content: String, pub speaker: String, pub speaker_id: i64,
    pub source: String, pub group_id: Option<i64>, pub message_time: String, pub created_at: String,
}

pub struct NoteStore(parking_lot::Mutex<Connection>);

impl NoteStore {
    pub fn new(data_dir: &str) -> Self {
        let dir = PathBuf::from(data_dir);
        let db = open_db(&dir.join("data.db").to_string_lossy());
        let path = dir.join("notes.json");
        if path.exists() {
            if let Ok(s) = std::fs::read_to_string(&path) {
                if let Ok(entries) = serde_json::from_str::<Vec<NoteEntry>>(&s) {
                    let conn = db.lock();
                    for e in &entries { conn.execute("INSERT OR IGNORE INTO notes VALUES(?1,?2,?3,?4,?5,?6,?7,?8)", params![e.id,e.content,e.speaker,e.speaker_id,e.source,e.group_id,e.message_time,e.created_at]).ok(); }
                }
            }
            let _ = std::fs::rename(&path, dir.join("notes.json.bak"));
        }
        Self(db)
    }

    pub fn create(&self, content: String, speaker: String, speaker_id: i64, source: String, group_id: Option<i64>, message_time: String) -> NoteEntry {
        let e = NoteEntry{id:uuid::Uuid::new_v4().to_string(),content,speaker,speaker_id,source,group_id,message_time,created_at:chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string()};
        self.0.lock().execute("INSERT OR REPLACE INTO notes VALUES(?1,?2,?3,?4,?5,?6,?7,?8)", params![e.id,e.content,e.speaker,e.speaker_id,e.source,e.group_id,e.message_time,e.created_at]).ok();
        e
    }

    pub fn list(&self) -> Vec<NoteEntry> {
        let conn = self.0.lock();
        let mut stmt = conn.prepare("SELECT id,content,speaker,speaker_id,source,group_id,message_time,created_at FROM notes ORDER BY rowid DESC").unwrap();
        let mut out = Vec::new();
        if let Ok(rows) = stmt.query_map([], |r| Ok(NoteEntry{id:r.get(0)?,content:r.get(1)?,speaker:r.get(2)?,speaker_id:r.get(3)?,source:r.get(4)?,group_id:r.get(5)?,message_time:r.get(6)?,created_at:r.get(7)?})) {
            for row in rows { if let Ok(e) = row { out.push(e); } }
    }
        out
    }
    pub fn delete(&self, id: &str) -> bool {
        self.0.lock().execute("DELETE FROM notes WHERE id=?1", params![id]).ok().is_some()
    }
}

// === Message History ===

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChatMessage { pub role: String, pub content: String, pub name: Option<String> }

pub struct MessageHistoryStore { history: Mutex<HashMap<String, VecDeque<ChatMessage>>>, max_per_chat: usize }

impl MessageHistoryStore {
    pub fn new(max_per_chat: usize) -> Self { Self { history: Mutex::new(HashMap::new()), max_per_chat } }
    pub fn push(&self, chat_id: &str, msg: ChatMessage) {
        let mut h = self.history.lock().unwrap();
        let e = h.entry(chat_id.to_string()).or_insert_with(|| VecDeque::with_capacity(self.max_per_chat));
        if e.len() >= self.max_per_chat { e.pop_front(); }
        e.push_back(msg);
    }
    pub fn recent(&self, chat_id: &str, count: usize) -> Vec<ChatMessage> {
        self.history.lock().unwrap().get(chat_id).map(|e| e.iter().rev().take(count).cloned().collect()).unwrap_or_default()
    }
    pub fn clear(&self, chat_id: &str) { self.history.lock().unwrap().remove(chat_id); }
}

// === Util ===

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum();
    let nb: f32 = b.iter().map(|x| x * x).sum();
    let denom = na.sqrt() * nb.sqrt();
    if denom < 1e-10 { 0.0 } else { dot / denom }
}

fn try_parse_time(s: &str) -> Option<String> {
    let now = chrono::Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    let tomorrow = (now + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s.trim(), "%Y-%m-%d %H:%M") { return Some(dt.format("%Y-%m-%d %H:%M").to_string()); }
    let hour = s.find('点').and_then(|pos| s[..pos].chars().rev().take(2).collect::<String>().chars().rev().collect::<String>().trim().parse::<u32>().ok());
    if let Some(mut h) = hour { if s.contains("下午")||s.contains("晚上") { if h < 12 { h += 12; } } return Some(format!("{} {:02}:00", if s.contains("明天"){&tomorrow}else{&today}, h)); }
    None
}

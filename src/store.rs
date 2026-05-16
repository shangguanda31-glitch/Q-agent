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
         CREATE TABLE IF NOT EXISTS notes(id TEXT PRIMARY KEY,content TEXT,speaker TEXT,speaker_id INTEGER,source TEXT,group_id INTEGER,message_time TEXT,created_at TEXT);
         CREATE TABLE IF NOT EXISTS exclusions(id INTEGER PRIMARY KEY AUTOINCREMENT,exclude_type TEXT NOT NULL,target_id INTEGER NOT NULL,note TEXT,created_at TEXT,UNIQUE(exclude_type,target_id));"
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

    pub fn load_context(&self, query_embedding: Option<&[f32]>, query: &str, max_entries: usize) -> String {
        let entries = self.read(query_embedding, query, max_entries);
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

// === Exclusion Store ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExclusionEntry {
    pub id: i64,
    pub exclude_type: String,
    pub target_id: i64,
    pub note: String,
    pub created_at: String,
}

pub struct ExclusionStore(parking_lot::Mutex<Connection>);

impl ExclusionStore {
    pub fn new(data_dir: &str) -> Self {
        let dir = PathBuf::from(data_dir);
        std::fs::create_dir_all(&dir).ok();
        let db = open_db(&dir.join("data.db").to_string_lossy());
        Self(db)
    }

    pub fn list(&self) -> Vec<ExclusionEntry> {
        let conn = self.0.lock();
        let mut stmt = conn.prepare("SELECT id,exclude_type,target_id,note,created_at FROM exclusions ORDER BY exclude_type,target_id").unwrap();
        let mut out = Vec::new();
        if let Ok(rows) = stmt.query_map([], |r| Ok(ExclusionEntry {
            id: r.get(0)?, exclude_type: r.get(1)?, target_id: r.get(2)?,
            note: get_str(r, 3), created_at: get_str(r, 4),
        })) {
            for row in rows { if let Ok(e) = row { out.push(e); } }
        }
        out
    }

    pub fn add(&self, exclude_type: &str, target_id: i64, note: &str) -> bool {
        self.0.lock().execute(
            "INSERT OR IGNORE INTO exclusions(exclude_type,target_id,note,created_at) VALUES(?1,?2,?3,?4)",
            params![exclude_type, target_id, note, chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string()],
        ).is_ok()
    }

    pub fn remove(&self, exclude_type: &str, target_id: i64) -> bool {
        self.0.lock().execute(
            "DELETE FROM exclusions WHERE exclude_type=?1 AND target_id=?2",
            params![exclude_type, target_id],
        ).is_ok()
    }

    pub fn is_excluded(&self, exclude_type: &str, target_id: i64) -> bool {
        self.0.lock().query_row(
            "SELECT COUNT(*) FROM exclusions WHERE exclude_type=?1 AND target_id=?2",
            params![exclude_type, target_id],
            |r| r.get::<_, i64>(0),
        ).unwrap_or(0) > 0
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
    use chrono::Datelike;
    let now = chrono::Local::now();
    let s = s.trim();

    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
        return Some(dt.format("%Y-%m-%d %H:%M").to_string());
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(format!("{} 00:00", d));
    }

    let day_lower = s.to_lowercase();

    // Base date: handle relative days
    let mut base_date = now.date_naive();
    if day_lower.contains("大后天") { base_date += chrono::Duration::days(3); }
    if day_lower.contains("后天") { base_date += chrono::Duration::days(2); }
    if day_lower.contains("明天") || day_lower.contains("明日") { base_date += chrono::Duration::days(1); }

    // Weekday references: 下周三 / 本周五 / 这周二
    let weekdays = [("一",0),("二",1),("三",2),("四",3),("五",4),("六",5),("日",6),("天",6)];
    for &(name, target) in &weekdays {
        let current = base_date.weekday().num_days_from_monday();
        if day_lower.contains(&format!("下{}", name)) {
            let ahead = if target > current { target - current + 7 } else { target - current + 14 };
            base_date += chrono::Duration::days(ahead as i64);
            break;
        }
        if day_lower.contains(&format!("本{}", name)) || day_lower.contains(&format!("这{}", name)) {
            let ahead = if target >= current { target - current } else { target - current + 7 };
            if ahead > 0 { base_date += chrono::Duration::days(ahead as i64); }
            break;
        }
    }

    // Month-day: X月X号 / X月X日
    if let Some((m, d)) = parse_month_day(s) {
        let y = base_date.year();
        if let Ok(date) = chrono::NaiveDate::parse_from_str(&format!("{}-{:02}-{:02}", y, m, d), "%Y-%m-%d") {
            if date < base_date {
                base_date = chrono::NaiveDate::parse_from_str(&format!("{}-{:02}-{:02}", y + 1, m, d), "%Y-%m-%d").unwrap();
            } else {
                base_date = date;
            }
        }
    }

    // Time parsing
    let is_pm = day_lower.contains("下午") || day_lower.contains("晚上") || day_lower.contains("傍晚");
    let is_am = day_lower.contains("上午") || day_lower.contains("早上") || day_lower.contains("凌晨");
    let is_noon = day_lower.contains("中午");

    let mut hour = None;
    let mut minute = None;

    if let Some(pos) = s.find('点') {
        let before: String = s[..pos].chars().filter(|c| c.is_ascii_digit()).collect();
        hour = before.parse::<u32>().ok();

        let after = &s[pos + 3..]; // skip '点' (UTF-8)
        if after.starts_with("半") { minute = Some(30); }
        else if after.starts_with("一刻") || after.starts_with('刻') { minute = Some(15); }
        else if after.starts_with("三刻") { minute = Some(45); }
        else {
            // "X分" or plain number
            let digit_str: String = after.chars().filter(|c| c.is_ascii_digit()).collect();
            if !digit_str.is_empty() {
                let val = digit_str.parse::<u32>().unwrap_or(0);
                if val < 60 { minute = Some(val); }
            }
        }
    }

    let mut h = hour.unwrap_or(12);
    if is_noon && h < 12 { /* noon stays */ }
    else if is_pm && h < 12 { h += 12; }
    else if is_am && h >= 12 { h -= 12; }
    h = h.min(23);

    Some(format!("{} {:02}:{:02}", base_date.format("%Y-%m-%d"), h, minute.unwrap_or(0).min(59)))
}

fn parse_month_day(s: &str) -> Option<(u32, u32)> {
    let normalized = s.replace("号", "月").replace("日", "月");
    let parts: Vec<&str> = normalized.split('月').collect();
    if parts.len() >= 3 {
        let month = parts[0].chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok()?;
        let day = parts[1].chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok()?;
        if (1..=12).contains(&month) && (1..=31).contains(&day) {
            return Some((month, day));
        }
    }
    None
}

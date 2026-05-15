use serde::{Deserialize, Serialize};

// === OneBot v11 Event Types ===

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OneBotEvent {
    Message(MessageEvent),
    Notice(NoticeEvent),
    Request(RequestEvent),
    Meta(MetaEvent),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageEvent {
    pub post_type: String,
    pub message_type: String,
    pub sub_type: Option<String>,
    pub user_id: i64,
    pub group_id: Option<i64>,
    pub message_id: Option<i64>,
    pub message_seq: Option<i64>,
    pub real_id: Option<i64>,
    pub sender: Option<Sender>,
    pub message: Vec<MessageSegment>,
    pub raw_message: Option<String>,
    pub time: Option<i64>,
    pub self_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Sender {
    pub user_id: Option<i64>,
    pub nickname: Option<String>,
    pub card: Option<String>,
    pub role: Option<String>,
    pub sex: Option<String>,
    pub age: Option<i32>,
    pub area: Option<String>,
    pub level: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageSegment {
    #[serde(rename = "type")]
    pub segment_type: String,
    pub data: Option<serde_json::Value>,
}

impl MessageSegment {
    pub fn text(&self) -> Option<String> {
        self.data.as_ref().and_then(|d| d.get("text")).and_then(|v| v.as_str().map(String::from))
    }

    pub fn image_url(&self) -> Option<String> {
        self.data.as_ref().and_then(|d| d.get("url")).and_then(|v| v.as_str().map(String::from))
    }

    pub fn image_file(&self) -> Option<String> {
        self.data.as_ref().and_then(|d| d.get("file")).and_then(|v| v.as_str().map(String::from))
    }

    pub fn file_url(&self) -> Option<String> {
        self.data.as_ref().and_then(|d| d.get("url")).and_then(|v| v.as_str().map(String::from))
    }

    pub fn file_name(&self) -> Option<String> {
        self.data.as_ref().and_then(|d| d.get("name")).and_then(|v| v.as_str().map(String::from))
    }

    pub fn file_size(&self) -> Option<i64> {
        self.data.as_ref().and_then(|d| d.get("size")).and_then(|v| v.as_i64())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NoticeEvent {
    pub post_type: String,
    pub notice_type: String,
    pub sub_type: Option<String>,
    pub user_id: Option<i64>,
    pub group_id: Option<i64>,
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestEvent {
    pub post_type: String,
    pub request_type: String,
    pub user_id: Option<i64>,
    pub group_id: Option<i64>,
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetaEvent {
    pub post_type: String,
    pub meta_event_type: String,
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
}

// === LLM Analysis Result ===

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LLMAnalysis {
    pub priority: String,
    pub summary: String,
    pub need_schedule: bool,
    pub schedule_info: Option<ScheduleInfo>,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScheduleInfo {
    pub title: String,
    pub time: Option<String>,
    pub description: Option<String>,
}

// === Processed Event ===

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessedEvent {
    pub time: String,
    pub message_type: String,
    pub group_id: Option<i64>,
    pub group_name: Option<String>,
    pub user_id: i64,
    pub sender_name: String,
    pub raw_text: String,
    pub has_image: bool,
    pub image_urls: Vec<String>,
    pub has_file: bool,
    pub file_name: Option<String>,
    pub analysis: Option<LLMAnalysis>,
    pub raw_json: String,
}

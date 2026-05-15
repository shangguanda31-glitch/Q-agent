use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use crate::store::ScheduleStore;
use super::traits::{Tool, ToolResult};

pub struct ScheduleTool {
    store: Arc<ScheduleStore>,
}

impl ScheduleTool {
    pub fn new(store: Arc<ScheduleStore>) -> Arc<Self> { Arc::new(Self { store }) }
}

#[async_trait]
impl Tool for ScheduleTool {
    fn name(&self) -> &str { "schedule_create" }

    fn description(&self) -> &str {
        "保存一条日程。把你要记住的日程信息用自然语言传进来即可，我会保存。"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "日程标题"},
                "info": {"type": "string", "description": "日程详细信息，如时间、地点、参与人等，用自然语言描述"}
            },
            "required": ["title"]
        })
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("日程").to_string();
        let info = args.get("info").and_then(|v| v.as_str()).map(String::from);
        let time = info.as_ref().and_then(|_| None); // LLM handles time in the text
        let entry = self.store.create(title, time, info,
            "QQ消息".to_string(), "LLM提取".to_string());
        ToolResult::ok(format!("日程已保存: {} (ID: {})", entry.title, entry.id))
    }
}

pub struct ScheduleListTool {
    store: Arc<ScheduleStore>,
}

impl ScheduleListTool {
    pub fn new(store: Arc<ScheduleStore>) -> Arc<Self> { Arc::new(Self { store }) }
}

#[async_trait]
impl Tool for ScheduleListTool {
    fn name(&self) -> &str { "schedule_list" }

    fn description(&self) -> &str {
        "查看所有已保存的日程列表。"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({"type": "object", "properties": {}})
    }

    async fn execute(&self, _args: Value) -> ToolResult {
        let entries = self.store.list();
        if entries.is_empty() { return ToolResult::ok("暂无日程"); }
        let lines: Vec<String> = entries.iter().enumerate().map(|(i, e)| {
            let status = if e.status == "done" { "✅" } else { "⏳" };
            let info = e.description.as_deref().unwrap_or("");
            format!("{}. {} {} {}", i+1, status, e.title, info)
        }).collect();
        ToolResult::ok(lines.join("\n"))
    }
}

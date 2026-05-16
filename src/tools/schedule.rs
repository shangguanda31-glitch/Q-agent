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
        "保存一条日程。把标题和提取到的时间单独传进来。"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "日程标题"},
                "time": {"type": "string", "description": "日程时间，例如明天下午5点"},
                "info": {"type": "string", "description": "补充信息，如地点、参与人等"}
            },
            "required": ["title"]
        })
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("日程").to_string();
        let time = args.get("time").and_then(|v| v.as_str()).map(String::from);
        let info = args.get("info").and_then(|v| v.as_str()).map(String::from);
        let entry = self.store.create(title, time, info,
            "QQ消息".to_string(), "LLM提取".to_string());
        ToolResult::ok(format!("日程已保存: {} (ID: {})", entry.title, entry.id))
    }
}

pub struct ScheduleUpdateTool {
    store: Arc<ScheduleStore>,
}

impl ScheduleUpdateTool {
    pub fn new(store: Arc<ScheduleStore>) -> Arc<Self> { Arc::new(Self { store }) }
}

#[async_trait]
impl Tool for ScheduleUpdateTool {
    fn name(&self) -> &str { "schedule_update" }

    fn description(&self) -> &str {
        "更新已有日程的信息。当后续消息补充了地点、参与人等详情时，用此工具将新信息追加到原有日程中。"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "要更新的日程标题（用于匹配）"},
                "time": {"type": "string", "description": "要更新的日程时间（用于匹配）"},
                "info": {"type": "string", "description": "要追加的补充信息，如地点"}
            },
            "required": ["title", "info"]
        })
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let time = args.get("time").and_then(|v| v.as_str()).map(String::from);
        let info = args.get("info").and_then(|v| v.as_str()).unwrap_or("");

        let entries = self.store.list();
        let entry = entries.iter().find(|e| {
            e.title == title && time.as_ref().map(|t| e.time.as_deref() == Some(t.as_str())).unwrap_or(true)
        }).cloned();

        match entry {
            Some(e) => {
                let new_desc = match (&e.description, info) {
                    (Some(old), "") => old.clone(),
                    (Some(old), new) => format!("{}；{}", old, new),
                    (None, "") => return ToolResult::ok("日程无新信息需要更新"),
                    (None, new) => new.to_string(),
                };
                self.store.update_description(&e.id, &new_desc);
                ToolResult::ok(format!("已更新日程「{}」：{}", e.title, info))
            }
            None => ToolResult::fail(format!("未找到匹配的日程「{}」", title))
        }
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

    fn description(&self) -> &str { "查看所有已保存的日程列表。" }

    fn parameters_schema(&self) -> Value { serde_json::json!({"type": "object", "properties": {}}) }

    async fn execute(&self, _args: Value) -> ToolResult {
        let entries = self.store.list();
        if entries.is_empty() { return ToolResult::ok("暂无日程"); }
        let lines: Vec<String> = entries.iter().enumerate().map(|(i, e)| {
            let status = if e.status == "done" { "✅" } else { "⏳" };
            let tm = e.time.as_deref().unwrap_or("时间待定");
            let desc = e.description.as_deref().unwrap_or("");
            format!("{}. {} {}【{}】{}", i+1, status, e.title, tm, desc)
        }).collect();
        ToolResult::ok(lines.join("\n"))
    }
}

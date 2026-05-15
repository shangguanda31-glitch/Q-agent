use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use crate::notify;
use super::traits::{Tool, ToolResult};

pub struct NotifyTool;

impl NotifyTool {
    pub fn new() -> Arc<Self> { Arc::new(Self) }
}

#[async_trait]
impl Tool for NotifyTool {
    fn name(&self) -> &str { "notify_user" }

    fn description(&self) -> &str {
        "向用户发送 Windows 桌面通知弹窗，用于告知用户需要关注的重要信息"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "通知标题，简洁明了"},
                "body": {"type": "string", "description": "通知正文"}
            },
            "required": ["title", "body"]
        })
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("QQ助理通知");
        let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
        notify::send_toast(title, body);
        ToolResult::ok(format!("已发送通知: {} - {}", title, body))
    }
}

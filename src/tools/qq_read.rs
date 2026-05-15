use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use crate::napcat::api::NapCatApi;
use super::traits::{Tool, ToolResult};

pub struct QQReadTool {
    api: Arc<NapCatApi>,
}

impl QQReadTool {
    pub fn new(api: Arc<NapCatApi>) -> Arc<Self> { Arc::new(Self { api }) }
}

#[async_trait]
impl Tool for QQReadTool {
    fn name(&self) -> &str { "qq_read" }

    fn description(&self) -> &str {
        "读取 QQ 聊天记录、群公告、群信息等。当用户询问历史消息或群信息时使用"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["recent_messages", "group_notices", "group_info", "friend_list"],
                    "description": "读取类型"
                },
                "group_id": {"type": "number", "description": "群号（群相关操作时必填）"},
                "count": {"type": "number", "description": "获取消息条数，默认20"}
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");

        match action {
            "group_notices" => {
                let group_id = match args.get("group_id").and_then(|v| v.as_i64()) {
                    Some(id) => id,
                    None => return ToolResult::fail("需要 group_id 参数"),
                };
                let notices = self.api.get_group_notices(group_id).await;
                let output = if notices.is_empty() {
                    "暂无群公告".to_string()
                } else {
                    let mut lines = Vec::new();
                    for (i, n) in notices.iter().enumerate() {
                        let title = n.get("title").and_then(|v| v.as_str()).unwrap_or("无标题");
                        let publisher = n.get("publisher_id").and_then(|v| v.as_str()).unwrap_or("未知");
                        lines.push(format!("{}. [{}] {}", i + 1, publisher, title));
                        if let Some(msg) = n.get("msg").and_then(|v| v.as_str()) {
                            lines.push(format!("   {}", msg));
                        }
                    }
                    lines.join("\n")
                };
                ToolResult::ok(output)
            }
            _ => ToolResult::fail(format!("不支持的操作: {}", action)),
        }
    }
}

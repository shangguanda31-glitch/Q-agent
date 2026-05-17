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
        "读取 QQ 聊天记录（群/私聊）、群公告、群列表、好友列表等"
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

            "recent_messages" => {
                let count = args.get("count").and_then(|v| v.as_i64()).unwrap_or(20).min(50) as i32;
                match args.get("group_id").and_then(|v| v.as_i64()) {
                    Some(gid) => {
                        let msgs = self.api.get_group_msg_history(gid, count).await;
                        if msgs.is_empty() { return ToolResult::ok("暂无消息记录"); }
                        let lines: Vec<String> = msgs.iter().map(|m| {
                            let sender = m.pointer("/sender/nickname").and_then(|v| v.as_str()).unwrap_or("未知");
                            let text = m.pointer("/message").and_then(|v| v.as_str()).unwrap_or("");
                            format!("[{}] {}", sender, text.chars().take(100).collect::<String>())
                        }).collect();
                        ToolResult::ok(lines.join("\n"))
                    }
                    None => {
                        let uid = match args.get("user_id").and_then(|v| v.as_i64()) {
                            Some(id) => id,
                            None => return ToolResult::fail("需要 group_id 或 user_id 参数"),
                        };
                        let msgs = self.api.get_friend_msg_history(uid, count).await;
                        if msgs.is_empty() { return ToolResult::ok("暂无消息记录"); }
                        let lines: Vec<String> = msgs.iter().map(|m| {
                            let text = m.pointer("/message").and_then(|v| v.as_str()).unwrap_or("");
                            format!("{}", text.chars().take(100).collect::<String>())
                        }).collect();
                        ToolResult::ok(lines.join("\n"))
                    }
                }
            }
            "friend_list" => {
                let friends = self.api.get_friend_list().await;
                if friends.is_empty() {
                    return ToolResult::ok("好友列表为空");
                }
                let lines: Vec<String> = friends.iter().enumerate().map(|(i, f)| {
                    let name = f.get("nickname").or_else(|| f.get("remark")).and_then(|v| v.as_str()).unwrap_or("未知");
                    let uid = f.get("user_id").and_then(|v| v.as_i64()).unwrap_or(0);
                    format!("{}. {} ({})", i + 1, name, uid)
                }).collect();
                ToolResult::ok(lines.join("\n"))
            }
            "group_info" => {
                let groups = self.api.get_group_list().await;
                if groups.is_empty() {
                    return ToolResult::ok("暂无群聊");
                }
                let lines: Vec<String> = groups.iter().enumerate().map(|(i, g)| {
                    let name = g.get("group_name").and_then(|v| v.as_str()).unwrap_or("未知群");
                    let gid = g.get("group_id").and_then(|v| v.as_i64()).unwrap_or(0);
                    format!("{}. {} ({})", i + 1, name, gid)
                }).collect();
                ToolResult::ok(lines.join("\n"))
            }
            _ => ToolResult::fail(format!("不支持的操作: {}", action)),
        }
    }
}

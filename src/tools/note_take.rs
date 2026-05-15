use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use crate::store::NoteStore;
use super::traits::{Tool, ToolResult};

pub struct NoteTakeTool {
    store: Arc<NoteStore>,
}

impl NoteTakeTool {
    pub fn new(store: Arc<NoteStore>) -> Arc<Self> { Arc::new(Self { store }) }
}

#[async_trait]
impl Tool for NoteTakeTool {
    fn name(&self) -> &str { "note_take" }

    fn description(&self) -> &str {
        "记笔记。当你听到重要的信息、要点、决定、承诺等需要记录的内容时，用这个工具记下来。"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "content": {"type": "string", "description": "笔记内容，完整记录重点信息"},
                "speaker": {"type": "string", "description": "说话人"},
                "source": {"type": "string", "description": "来源，如'群聊'或'私聊'"}
            },
            "required": ["content"]
        })
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
        if content.is_empty() { return ToolResult::fail("笔记内容不能为空"); }
        let speaker = args.get("speaker").and_then(|v| v.as_str()).unwrap_or("未知");
        let source = args.get("source").and_then(|v| v.as_str()).unwrap_or("未知");
        // Note: speaker_id, group_id, message_time are auto-populated by the agent
        let entry = self.store.create(
            content.to_string(), speaker.to_string(), 0,
            source.to_string(), None, String::new(),
        );
        ToolResult::ok(format!("笔记已保存: {} (来自{})", &entry.content.chars().take(50).collect::<String>(), speaker))
    }
}

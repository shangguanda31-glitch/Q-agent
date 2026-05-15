use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use crate::store::MemoryStore;
use super::traits::{Tool, ToolResult};

pub struct RememberTool {
    store: Arc<MemoryStore>,
}

impl RememberTool {
    pub fn new(store: Arc<MemoryStore>) -> Arc<Self> { Arc::new(Self { store }) }
}

#[async_trait]
impl Tool for RememberTool {
    fn name(&self) -> &str { "remember" }

    fn description(&self) -> &str {
        "记住一条信息。把你想记住的内容用自然语言传进来，以后可以用 recall 回忆。用于保存用户的重要信息、偏好、约定等。"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "content": {"type": "string", "description": "要记住的内容，自然语言描述"}
            },
            "required": ["content"]
        })
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
        if content.is_empty() { return ToolResult::fail("内容不能为空"); }
        self.store.write(content.to_string(), vec!["user".to_string()], "llm".to_string());
        ToolResult::ok(format!("已记住：{}", content))
    }
}

pub struct RecallTool {
    store: Arc<MemoryStore>,
}

impl RecallTool {
    pub fn new(store: Arc<MemoryStore>) -> Arc<Self> { Arc::new(Self { store }) }
}

#[async_trait]
impl Tool for RecallTool {
    fn name(&self) -> &str { "recall" }

    fn description(&self) -> &str {
        "回忆之前记住的信息。用关键词搜索即可。"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "搜索关键词"}
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        if query.is_empty() { return ToolResult::fail("查询词不能为空"); }
        let results = self.store.read(query, 10);
        if results.is_empty() { return ToolResult::ok("未找到相关记忆"); }
        let output: Vec<String> = results.iter().map(|e| format!("- {}", e.content)).collect();
        ToolResult::ok(format!("找到 {} 条：\n{}", results.len(), output.join("\n")))
    }
}

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Semaphore;
use tracing::warn;

use super::traits::{Tool, ToolResult};

pub struct ClaudeCodeTool {
    timeout_secs: u64,
    max_output_bytes: usize,
    working_dir: String,
    queue: Semaphore,
}

impl ClaudeCodeTool {
    pub fn new(timeout_secs: u64, max_output_bytes: usize, working_dir: &str) -> Arc<Self> {
        Arc::new(Self { timeout_secs, max_output_bytes, working_dir: working_dir.to_string(), queue: Semaphore::new(2) })
    }
}

#[async_trait]
impl Tool for ClaudeCodeTool {
    fn name(&self) -> &str { "claude_code" }

    fn description(&self) -> &str {
        "将任务发送给 Claude Code CLI 处理。Claude Code 可以：写文档/PPT、编程、分析数据、修bug、写脚本、回答问题等。当你收到任何需要动手完成的任务时使用，特别是内容生成、代码开发、问题分析等"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "要提交给 Claude Code 的任务描述，包含完整的上下文和具体要求"
                }
            },
            "required": ["prompt"]
        })
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let _permit = self.queue.acquire().await.expect("semaphore closed");
        let prompt = match args.get("prompt").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::fail("需要 prompt 参数"),
        };

        let safe_prompt = prompt.replace('"', "\\\"");

        let wrapped = format!(
            "请完成以下任务，并在工作目录中创建实际的成果文件（如.docx、.pptx、.md、.html等）。\n\
             注意：必须输出完整内容，不要只写大纲。\n\n{}",
            safe_prompt
        );

        let mut child = match Command::new("claude")
            .args(["-p", &wrapped, "--dangerously-skip-permissions",
                   "--output-format", "stream-json", "--include-partial-messages", "--verbose",
                   "--effort", "max"])
            .current_dir(&self.working_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return ToolResult::fail(
                "无法启动 Claude Code CLI。请确保已安装 `claude` 命令（npm install -g @anthropic-ai/claude-code）"
            ),
        };

        let child_stdout = child.stdout.take().expect("stdout piped");
        let reader = BufReader::new(child_stdout);
        let mut lines = reader.lines();

        let start = std::time::Instant::now();
        let mut last_notify = std::time::Instant::now();
        let mut final_text = String::new();
        let mut thinking_buf = String::new();

        let timeout_duration = std::time::Duration::from_secs(self.timeout_secs);

        loop {
            let line = tokio::time::timeout(timeout_duration, lines.next_line()).await;
            let line = match line {
                Ok(Ok(Some(l))) => l,
                Ok(Ok(None)) => break, // EOF
                Ok(Err(e)) => {
                    // stdout read error (e.g. broken pipe) - process may have exited
                    break;
                }
                Err(_) => {
                    // Overall timeout
                    let _ = child.kill().await;
                    return ToolResult::fail(format!("Claude Code 执行超时 ({}s)\n部分输出:\n{}",
                        self.timeout_secs, truncate(&final_text, self.max_output_bytes)));
                }
            };

            if let Ok(json) = serde_json::from_str::<Value>(&line) {
                let typ = json.get("type").and_then(|v| v.as_str()).unwrap_or("");

                match typ {
                    "stream_event" => {
                        if let Some(event) = json.get("event") {
                            let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            match event_type {
                                "content_block_delta" => {
                                    if let Some(delta) = event.get("delta") {
                                        if let Some(think) = delta.get("thinking").and_then(|v| v.as_str()) {
                                            thinking_buf.push_str(think);
                                            // Notify when we have meaningful thinking
                                            if last_notify.elapsed() >= std::time::Duration::from_secs(10)
                                                && thinking_buf.len() > 20 {
                                                let preview: String = thinking_buf.chars().take(60).collect();
                                                crate::notify::send_toast(
                                                    &format!("Claude Code {}s", start.elapsed().as_secs()),
                                                    &preview);
                                                last_notify = std::time::Instant::now();
                                                thinking_buf.clear();
                                            }
                                        }
                                    }
                                }
                                "tool_use_start" => {
                                    let name = event.get("name").and_then(|v| v.as_str()).unwrap_or("tool");
                                    crate::notify::send_toast(
                                        &format!("Claude Code {}s", start.elapsed().as_secs()),
                                        &format!("正在{}...", name));
                                    last_notify = std::time::Instant::now();
                                    // Flush thinking on tool use
                                    if !thinking_buf.is_empty() && thinking_buf.len() > 20 {
                                        thinking_buf.clear();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    "assistant" => {
                        // Final result
                        if let Some(content) = json.pointer("/message/content") {
                            if let Some(parts) = content.as_array() {
                                for part in parts {
                                    if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                                        final_text.push_str(text);
                                    }
                                }
                            }
                        }
                        // Also check top-level text field
                        if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
                            final_text.push_str(text);
                        }
                    }
                    "system" => {
                        // System events (init, status, etc.)
                    }
                    _ => {}
                }
            }
        }

        let _ = child.wait().await;

        if final_text.is_empty() {
            final_text = "任务完成（无文本输出）".to_string();
        }

        let workspace = &self.working_dir;
        let file_list = match std::fs::read_dir(workspace) {
            Ok(entries) => {
                let files: Vec<String> = entries.filter_map(|e| e.ok()).filter(|e| e.path().is_file())
                    .map(|e| e.file_name().to_string_lossy().to_string()).collect();
                if files.is_empty() { String::new() }
                else { format!("\n\n创建工作区文件：{}", files.join(", ")) }
            }
            Err(_) => String::new()
        };

        let output = format!("{}{}\n\n---\n任务已完成", truncate(&final_text, self.max_output_bytes), file_list);
        ToolResult::ok(output)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...\n(输出已截断)", &s[..max])
    } else {
        s.to_string()
    }
}


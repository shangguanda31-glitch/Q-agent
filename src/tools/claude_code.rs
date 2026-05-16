use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::process::Command;
use tracing::warn;

use super::traits::{Tool, ToolResult};

pub struct ClaudeCodeTool {
    timeout_secs: u64,
    max_output_bytes: usize,
    working_dir: String,
}

impl ClaudeCodeTool {
    pub fn new(timeout_secs: u64, max_output_bytes: usize, working_dir: &str) -> Arc<Self> {
        Arc::new(Self { timeout_secs, max_output_bytes, working_dir: working_dir.to_string() })
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
        let prompt = match args.get("prompt").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::fail("需要 prompt 参数"),
        };

        let safe_prompt = prompt.replace('"', "\\\"");

        // Wrap prompt to ensure Claude Code creates actual files, not just text output
        let wrapped = format!(
            "请完成以下任务，并在工作目录中创建实际的成果文件（如.docx、.pptx、.md、.html等）。\n\
             注意：必须输出完整内容，不要只写大纲。\n\n{}",
            safe_prompt
        );
        let Ok(child) = Command::new("claude")
            .args(["-p", &wrapped, "--dangerously-skip-permissions", "--print", "--effort", "max"])
            .current_dir(&self.working_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        else {
            return ToolResult::fail(
                "无法启动 Claude Code CLI。请确保已安装 `claude` 命令（npm install -g @anthropic-ai/claude-code）"
            );
        };

        let result = match tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            child.wait_with_output(),
        ).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => return ToolResult::fail(format!("Claude Code 执行错误: {}", e)),
            Err(_) => return ToolResult::fail(format!("Claude Code 执行超时 ({}s)", self.timeout_secs)),
        };

        let mut stdout = String::from_utf8_lossy(&result.stdout).to_string();
        let _stderr = String::from_utf8_lossy(&result.stderr);

        if stdout.len() > self.max_output_bytes {
            stdout.truncate(self.max_output_bytes);
            stdout.push_str("\n...(输出已截断)");
        }

        if result.status.success() {
            // Include file listing in workspace
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
            ToolResult::ok(format!("{}{}", stdout, file_list))
        } else {
            warn!("Claude Code returned non-zero exit: {}", result.status);
            ToolResult::fail(format!("Claude Code 返回错误 ({}):\n{}", result.status, stdout))
        }
    }
}

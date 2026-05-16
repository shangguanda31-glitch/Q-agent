use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
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
        Arc::new(Self { timeout_secs, max_output_bytes, working_dir: working_dir.to_string(), queue: Semaphore::new(1) })
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
        // Queue: only one claude_code runs at a time
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
            .args(["-p", &wrapped, "--dangerously-skip-permissions", "--print", "--effort", "max"])
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

        let mut child_stdout = child.stdout.take().expect("stdout piped");
        let child_stderr = child.stderr.take().expect("stderr piped");

        // Write progress to a file that the web panel can read
        let progress_file = format!("{}/.claude_progress", self.working_dir);
        let _ = tokio::fs::write(&progress_file, "启动 Claude Code...").await;

        // Background task: read stderr line by line for progress
        let pf = progress_file.clone();
        let stderr_task = tokio::spawn(async move {
            let reader = BufReader::new(child_stderr);
            let mut lines = reader.lines();
            let mut last_notify = std::time::Instant::now();

            while let Ok(Some(line)) = lines.next_line().await {
                let clean = line.trim().to_string();
                if !clean.is_empty() && !clean.contains('\x1b') {
                    let _ = tokio::fs::write(&pf, &clean).await;
                    if last_notify.elapsed() >= std::time::Duration::from_secs(20) {
                        let preview: String = clean.chars().take(50).collect();
                        crate::notify::send_toast("Claude Code 处理中", &preview);
                        last_notify = std::time::Instant::now();
                    }
                }
            }
        });

        // Read stdout for the final response
        let timeout_duration = std::time::Duration::from_secs(self.timeout_secs);
        let mut output_buf = Vec::new();
        let read_result = tokio::time::timeout(timeout_duration, child_stdout.read_to_end(&mut output_buf)).await;

        // Wait for process exit
        let status = child.wait().await.ok();

        // Clean up progress file
        stderr_task.abort();
        let _ = tokio::fs::write(&progress_file, "").await;

        let mut stdout = String::from_utf8_lossy(&output_buf).to_string();

        if stdout.len() > self.max_output_bytes {
            stdout.truncate(self.max_output_bytes);
            stdout.push_str("\n...(输出已截断)");
        }

        // Handle timeout vs completion
        if read_result.is_err() {
            return ToolResult::fail(format!("Claude Code 执行超时 ({}s)\n部分输出:\n{}", self.timeout_secs, stdout));
        }

        let success = status.map(|s| s.success()).unwrap_or(false);

        if success {
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
            warn!("Claude Code returned non-zero exit");
            ToolResult::fail(format!("Claude Code 返回错误:\n{}", stdout))
        }
    }
}

mod dispatcher;
mod prompt;

use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

use base64::Engine;
use crate::llm::{LLMClient, AgentMessage};
use crate::napcat::api::NapCatApi;
use crate::napcat::types::{OneBotEvent, ProcessedEvent, MessageEvent};
use crate::notify;
use crate::store::{EventStore, ScheduleStore, ChatMessage, MemoryStore};
use crate::tools::traits::ToolRegistry;

pub async fn run(
    mut rx: broadcast::Receiver<OneBotEvent>,
    llm: Arc<LLMClient>,
    _napcat_api: Arc<NapCatApi>,
    processed_tx: broadcast::Sender<ProcessedEvent>,
    event_store: Arc<EventStore>,
    _schedule_store: Arc<ScheduleStore>,
    tools: Arc<ToolRegistry>,
    max_iterations: usize,
    chat_history: Arc<crate::store::MessageHistoryStore>,
    memory_store: Arc<MemoryStore>,
) {
    loop {
        match rx.recv().await {
            Ok(event) => {
                let llm = llm.clone();
                let api = _napcat_api.clone();
                let pt = processed_tx.clone();
                let es = event_store.clone();
                let ss = _schedule_store.clone();
                let tools = tools.clone();
                let ch = chat_history.clone();
                let ms = memory_store.clone();
                tokio::spawn(async move {
                    if let OneBotEvent::Message(msg) = event {
                        handle_message(msg, llm, api, pt, es, ss, tools, max_iterations, ch, ms).await;
                    }
                });
            }
            Err(broadcast::error::RecvError::Lagged(n)) => warn!("Agent lagged {} messages", n),
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

async fn handle_message(
    msg: MessageEvent,
    llm: Arc<LLMClient>,
    napcat_api: Arc<NapCatApi>,
    processed_tx: broadcast::Sender<ProcessedEvent>,
    event_store: Arc<EventStore>,
    _schedule_store: Arc<ScheduleStore>,
    tools: Arc<ToolRegistry>,
    max_iterations: usize,
    chat_history: Arc<crate::store::MessageHistoryStore>,
    memory_store: Arc<MemoryStore>,
) {
    let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let sender_name = msg.sender.as_ref()
        .and_then(|s| s.card.as_deref().or(s.nickname.as_deref()))
        .unwrap_or("").to_string();
    let sender_name = if sender_name.is_empty() { msg.user_id.to_string() } else { sender_name };

    // Build message text description
    let mut text_parts = Vec::new();
    let mut has_image = false;
    let mut file_name = None;
    let mut is_mentioned = false;

    for seg in &msg.message {
        match seg.segment_type.as_str() {
            "text" => { if let Some(t) = seg.text() { text_parts.push(t); } }
            "image" => { has_image = true; text_parts.push("[图片]".to_string()); }
            "file" => {
                file_name = seg.file_name().or_else(|| {
                    seg.data.as_ref().and_then(|d| d.get("file_id")).and_then(|v| v.as_str().map(String::from))
                        .or_else(|| seg.data.as_ref().and_then(|d| d.get("fid")).and_then(|v| v.as_str().map(String::from)))
                });
                let fname = file_name.as_deref().or_else(|| {
                    seg.data.as_ref().and_then(|d| d.get("file_id")).and_then(|v| v.as_str())
                }).unwrap_or("文件");
                text_parts.push(format!("[{}]", fname));
            }
            "reply" => text_parts.push("[回复消息]".to_string()),
            "at" => {
                if let Some(qq) = seg.data.as_ref().and_then(|d| d.get("qq")).and_then(|v| v.as_str()) {
                    if let Some(self_id) = msg.self_id {
                        if qq == &self_id.to_string() || qq == "all" {
                            is_mentioned = true;
                        }
                    }
                }
                text_parts.push("[@]".to_string());
            }
            "face" | "mface" => text_parts.push("[表情]".to_string()),
            _ => text_parts.push(format!("[{}]", seg.segment_type)),
        }
    }
    let raw_text = text_parts.join(" ");

    let source = match (&msg.message_type[..], msg.group_id) {
        ("group", Some(gid)) => format!("群 {}", gid),
        ("private", _) => "私聊".to_string(),
        _ => "未知".to_string(),
    };
    let chat_id = msg.group_id.map(|g| format!("group_{}", g)).unwrap_or_else(|| format!("private_{}", msg.user_id));

    // Build the user-facing message prompt
    let mention_prefix = if is_mentioned { "\n⚠️ 这条消息提到了你（@了你），需要关注！" } else { "" };
    let user_prompt = format!(
        "消息来源: {}\n发送者: {}({})\n消息内容: {}{}",
        source, sender_name, msg.user_id, raw_text, mention_prefix,
    );

    // First do a quick classification (existing method)
    // Actually use agent loop directly - LLM will decide
    info!("Agent processing message from {}: {}", sender_name, &raw_text.chars().take(80).collect::<String>());

    // Load relevant memories and recent chat history into context
    let memory_context = memory_store.load_context(&raw_text, 10);
    let system_prompt = crate::agent::prompt::build_system_prompt(&tools, &memory_context);

    let mut messages: Vec<AgentMessage> = Vec::new();
    // Prepend recent chat history for continuous context
    let recent_history = chat_history.recent(&chat_id, 20);
    for h in recent_history.iter().rev() {
        messages.push(AgentMessage {
            role: h.role.clone(),
            content: h.content.clone(),
        });
    }
    messages.push(AgentMessage { role: "user".to_string(), content: user_prompt.clone() });

    let mut final_response = String::new();

    for iteration in 0..max_iterations {
        // Context window management: summarize old messages instead of trimming
        const MAX_TOKENS: usize = 6144;
        let est_tokens: usize = system_prompt.len() / 4 + messages.iter().map(|m| m.content.len() / 4 + 10).sum::<usize>();

        if est_tokens > MAX_TOKENS && messages.len() > 6 {
            // Keep: summary + system_prompt_equivalent + latest 4 messages
            let split = messages.len().saturating_sub(4);
            let old_msgs = messages.drain(..split).collect::<Vec<_>>();

            // Summarize old messages using LLM
            let old_text: String = old_msgs.iter()
                .map(|m| format!("[{}]: {}", m.role, m.content.chars().take(200).collect::<String>()))
                .collect::<Vec<_>>().join("\n").chars().take(3000).collect();

            let summary_prompt = format!(
                "以下是QQ群聊/私聊的对话历史，请用中文总结关键信息（讨论了什么、谁说了什么重要内容、做出了什么决定）。\
                 保持客观，不要遗漏重要事实。\n\n对话历史：\n{}", old_text);

            match llm.agent_chat(&[], "你是一个对话摘要助手。用简洁的语言总结对话要点，不超过200字。", None).await {
                Ok(summary_text) => {
                    // Try to get a summary from LLM
                    let summary_body = format!("[对话摘要]\n{}", summary_text);
                    messages.insert(0, AgentMessage { role: "user".to_string(), content: summary_body });
                    info!("Context summary created (replaced {} messages)", split);
                }
                Err(_) => {
                    // Fallback: simple keyword extraction as summary
                    let keywords: String = old_msgs.iter()
                        .flat_map(|m| m.content.split_whitespace())
                        .filter(|w| w.len() > 2)
                        .collect::<Vec<_>>().join(" ");
                    let simple_summary = format!("[历史摘要] 之前聊到了：{}", &keywords.chars().take(200).collect::<String>());
                    messages.insert(0, AgentMessage { role: "user".to_string(), content: simple_summary });
                    info!("Context fallback summary (LLM failed, used keyword extract)");
                }
            }
        }

        // Download images: for vision LLM and OCR
        let mut image_b64: Option<String> = None;
        let mut image_path: Option<String> = None;
        if has_image {
            let _ = std::fs::create_dir_all("image_cache");
            let mut idx = 0;
            for seg in &msg.message {
                if seg.segment_type == "image" {
                    if let Some(url) = seg.image_url() {
                        if let Some(data) = napcat_api.download_file(&url).await {
                            let filename = format!("img_{}_{}.jpg", msg.user_id, idx);
                            let local = format!("image_cache/{}", filename);
                            let _ = std::fs::write(&local, &data);
                            if image_b64.is_none() {
                                image_b64 = Some(base64::engine::general_purpose::STANDARD.encode(&data));
                                image_path = Some(local);
                            }
                            idx += 1;
                        }
                    }
                }
            }
        }

        // Skip LLM vision (no mmproj), inject OCR hint for images
        if has_image {
            if let Some(ref path) = image_path {
                let hint = format!("{} [此消息包含图片，已保存到 {}。如需识别图片中的文字，请调用 ocr_image 工具，参数 image_path 设为此路径]",
                    messages.last().map(|m| m.content.as_str()).unwrap_or(""), path);
                if let Some(last) = messages.last_mut() { last.content = hint; }
            }
        }
        let response = {
            match llm.agent_chat(&messages, &system_prompt, None).await {
                Ok(r) => r,
                Err(e) => { warn!("LLM agent error: {}", e); final_response = format!("LLM 错误: {}", e); break; }
            }
        };

        let (text, tool_calls) = dispatcher::parse_tool_calls(&response);

        if tool_calls.is_empty() {
            let preview: String = text.chars().take(60).collect();
            info!("LLM decided: no tool calls. Response: {}", preview);
            final_response = text;
            break;
        }

        info!("Agent iteration {}: {} tool call(s)", iteration + 1, tool_calls.len());
        final_response = text;

        // Execute each tool call
        for tc in &tool_calls {
            let tool_name = &tc.name;
            match tools.get(tool_name) {
                Some(tool) => {
                    info!("Executing tool: {} with args: {}", tool_name, tc.arguments);
                    let result = tool.execute(tc.arguments.clone()).await;

                    // If schedule tool was called and succeeded, also notify user
                    if tool_name == "schedule" && result.success {
                        notify::send_toast("日程已更新", &result.output);
                    }
                    if tool_name == "claude_code" && result.success {
                        let preview: String = result.output.chars().take(200).collect();
                        notify::send_toast("Claude Code 已完成", &preview);
                    }

                    let result_text = if result.success {
                        format!("工具 {} 执行成功:\n{}", tool_name, result.output)
                    } else {
                        format!("工具 {} 执行失败:\n{}", tool_name, result.output)
                    };
                    messages.push(AgentMessage { role: "user".to_string(), content: format!("<tool_result name=\"{}\">\n{}\n</tool_result>", tool_name, result_text) });
                }
                None => {
                    warn!("Unknown tool: {}", tool_name);
                    messages.push(AgentMessage { role: "user".to_string(), content: format!("Unknown tool: {}", tool_name) });
                }
            }
        }
    }

    if final_response.is_empty() {
        final_response = "消息已接收".to_string();
    }

    // Store chat history
    chat_history.push(&chat_id, ChatMessage {
        role: "user".to_string(), content: user_prompt.clone(), name: None,
    });
    chat_history.push(&chat_id, ChatMessage {
        role: "assistant".to_string(), content: final_response.clone(), name: None,
    });

    let processed = ProcessedEvent {
        time,
        message_type: msg.message_type.clone(),
        group_id: msg.group_id,
        group_name: msg.group_id.map(|g| g.to_string()),
        user_id: msg.user_id,
        sender_name,
        raw_text,
        has_image,
        image_urls: vec![],
        has_file: file_name.is_some(),
        file_name,
        analysis: None,
        raw_json: serde_json::to_string(&msg).unwrap_or_default(),
    };

    event_store.push(processed.clone());
    let _ = processed_tx.send(processed);
}

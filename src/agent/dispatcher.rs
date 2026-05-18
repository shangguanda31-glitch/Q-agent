use serde::Deserialize;
use tracing::warn;

#[derive(Debug, Clone, Deserialize)]
pub struct ParsedToolCall {
    #[serde(alias = "tool")]
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Parse LLM response: strip <think> blocks, extract <tool_call> tags.
/// Returns (cleaned_text, vec_of_tool_calls).
pub fn parse_tool_calls(response: &str) -> (String, Vec<ParsedToolCall>) {
    let mut text = response.to_string();

    // Strip <think>...</think> blocks (Qwen reasoning)
    while let (Some(start), Some(end)) = (text.find("<think>"), text.find("</think>")) {
        text.replace_range(start..=end + 8, "");
    }

    // Extract <tool_call>...</tool_call> blocks
    let mut calls = Vec::new();
    let mut cleaned = String::new();
    let mut remaining = text.as_str();
    let mut parse_errors = 0u32;

    while let Some(start) = remaining.find("<tool_call>") {
        cleaned.push_str(&remaining[..start]);
        remaining = &remaining[start + 11..];

        if let Some(end) = remaining.find("</tool_call>") {
            let json_str = remaining[..end].trim();
            match serde_json::from_str::<ParsedToolCall>(json_str) {
                Ok(call) => calls.push(call),
                Err(e) => {
                    parse_errors += 1;
                    if parse_errors <= 3 {
                        warn!("Failed to parse tool_call JSON (attempt {}): {} | content: {}",
                              parse_errors, e, json_str.chars().take(80).collect::<String>());
                    }
                }
            }
            remaining = &remaining[end + 12..];
        } else {
            warn!("Unclosed <tool_call> tag (no </tool_call> found)");
            break;
        }
    }
    cleaned.push_str(remaining);

    // Fallback: if no XML tags found, try to parse entire response as bare JSON tool call
    if calls.is_empty() {
        let trimmed = response.trim();
        // Try {"name":"...", "arguments":{...}} or {"tool":"...", "arguments":{...}}
        if let Ok(mut call) = serde_json::from_str::<ParsedToolCall>(trimmed) {
            if !call.name.is_empty() {
                normalize_args(&mut call.arguments);
                calls.push(call);
                return (String::new(), calls);
            }
        }
        // Try {"tool_call": {"name": "...", "arguments": {...}}}
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(tc) = val.get("tool_call") {
                if let Ok(mut call) = serde_json::from_value::<ParsedToolCall>(tc.clone()) {
                    if !call.name.is_empty() {
                        normalize_args(&mut call.arguments);
                        calls.push(call);
                        return (String::new(), calls);
                    }
                }
            }
        }
    }

    (cleaned.trim().to_string(), calls)
}

/// Normalize tool argument field names for model compatibility.
/// Maps common aliases across different LLM backends to canonical names.
fn normalize_args(args: &mut serde_json::Value) {
    let aliases: &[(&[&str], &str)] = &[
        (&["event", "event_name", "task", "subject"], "title"),
        (&["date", "datetime", "when", "start_time"], "time"),
        (&["description", "details", "note", "content"], "info"),
        (&["text", "message", "msg"], "content"),
        (&["query", "search", "keyword", "question"], "query"),
        (&["person", "who", "user", "target"], "name"),
    ];

    if let Some(obj) = args.as_object_mut() {
        for &(from_list, to) in aliases {
            if obj.contains_key(to) { continue; } // already has canonical key
            for &from in from_list {
                if let Some(val) = obj.remove(from) {
                    obj.insert(to.to_string(), val);
                    break;
                }
            }
        }
    }
}

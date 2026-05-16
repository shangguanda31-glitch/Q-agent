use serde::Deserialize;
use tracing::warn;

#[derive(Debug, Clone, Deserialize)]
pub struct ParsedToolCall {
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

    (cleaned.trim().to_string(), calls)
}

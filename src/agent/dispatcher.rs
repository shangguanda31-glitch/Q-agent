use serde::Deserialize;

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

    while let Some(start) = remaining.find("<tool_call>") {
        // Everything before this tag is plain text
        cleaned.push_str(&remaining[..start]);
        remaining = &remaining[start + 11..]; // skip <tool_call>

        if let Some(end) = remaining.find("</tool_call>") {
            let json_str = remaining[..end].trim();
            if let Ok(call) = serde_json::from_str::<ParsedToolCall>(json_str) {
                calls.push(call);
            }
            remaining = &remaining[end + 12..]; // skip </tool_call>
        } else {
            break;
        }
    }
    cleaned.push_str(remaining);

    (cleaned.trim().to_string(), calls)
}

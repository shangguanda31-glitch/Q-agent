use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentMessage {
    pub role: String,
    pub content: String,
}

pub struct LLMClient {
    client: reqwest::Client,
    base_url: String,
    embed_url: String,
    model: String,
}

impl LLMClient {
    pub fn new(
        base_url: &str,
        embed_url: &str,
        model: &str,
        connect_timeout_secs: u64,
        read_timeout_secs: u64,
    ) -> Self {
        Self {
            client: reqwest::Client::builder()
                .no_proxy()
                .connect_timeout(Duration::from_secs(connect_timeout_secs))
                .read_timeout(Duration::from_secs(read_timeout_secs))
                .build()
                .expect("Failed to build HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
            embed_url: embed_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
        }
    }

    pub async fn agent_chat(
        &self,
        messages: &[AgentMessage],
        system_prompt: &str,
        image_b64: Option<&str>,
    ) -> anyhow::Result<String> {
        let mut body_parts: Vec<Value> = vec![
            serde_json::json!({"role": "system", "content": system_prompt}),
        ];
        for (i, msg) in messages.iter().enumerate() {
            if i == messages.len().saturating_sub(1) && msg.role == "user" && image_b64.is_some() {
                body_parts.push(serde_json::json!({
                    "role": "user",
                    "content": [
                        {"type": "text", "text": msg.content},
                        {"type": "image_url", "image_url": {"url": format!("data:image/jpeg;base64,{}", image_b64.unwrap())}}
                    ]
                }));
            } else {
                body_parts.push(serde_json::json!({"role": msg.role, "content": msg.content}));
            }
        }

        let body = serde_json::json!({
            "model": self.model,
            "messages": body_parts,
            "temperature": 0.3,
            "max_tokens": 4096,
        });

        let resp = self.client.post(format!("{}/v1/chat/completions", self.base_url)).json(&body).send().await?;
        let status = resp.status();
        let resp_text = resp.text().await.unwrap_or_default();

        if !status.is_success() {
            anyhow::bail!("LLM API returned {}: {}", status, resp_text.chars().take(500).collect::<String>());
        }

        let result: Value = serde_json::from_str(&resp_text)
            .map_err(|e| anyhow::anyhow!("LLM JSON parse error: {} | body: {}", e, resp_text.chars().take(500).collect::<String>()))?;

        let content = result.pointer("/choices/0/message/content").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Agent LLM returned no content: {}", resp_text.chars().take(500).collect::<String>()))?;

        Ok(content.trim().to_string())
    }

    /// Generate embedding vector for text via /v1/embeddings
    pub async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let body = serde_json::json!({
            "model": self.model,
            "input": text,
        });
        let resp = self.client.post(format!("{}/v1/embeddings", self.embed_url)).json(&body).send().await?;
        let status = resp.status();
        let resp_text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("Embedding API returned {}: {}", status, resp_text.chars().take(200).collect::<String>());
        }
        let result: serde_json::Value = serde_json::from_str(&resp_text)?;
        let data = result.pointer("/data/0/embedding")
            .ok_or_else(|| anyhow::anyhow!("No embedding in response"))?;
        let emb: Vec<f32> = serde_json::from_value(data.clone())?;
        Ok(emb)
    }
}

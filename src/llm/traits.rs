use async_trait::async_trait;

/// A message in the LLM conversation
#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentMessage {
    pub role: String,
    pub content: String,
}

/// Abstract LLM provider trait.
/// Every backend (OpenAI-compatible, Anthropic, Ollama, ...) implements this.
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Chat completion with optional base64 image.
    async fn chat(&self, messages: &[AgentMessage], system_prompt: &str, image_b64: Option<&str>) -> anyhow::Result<String>;

    /// Generate embedding vector for text.
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
}

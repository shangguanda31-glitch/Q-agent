use std::sync::Arc;

use super::openai::OpenAIProvider;
use super::traits::LLMProvider;
use crate::config::LLMBackend;

/// Create the appropriate LLM provider based on configuration.
pub fn create_provider(backend: &LLMBackend, llm_url: &str, embed_url: &str, model: &str) -> Arc<dyn LLMProvider> {
    match backend {
        LLMBackend::OpenAI => {
            Arc::new(OpenAIProvider::new(llm_url, embed_url, model))
        }
        // Future backends:
        // LLMBackend::Anthropic => Arc::new(AnthropicProvider::new(...)),
        // LLMBackend::Ollama => Arc::new(OpenAIProvider::new(...)),  // Ollama also speaks OpenAI API
    }
}

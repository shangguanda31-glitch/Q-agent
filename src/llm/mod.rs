//! LLM 抽象层 — 多后端统一接口

mod factory;
mod openai;
mod traits;

pub use traits::{AgentMessage, LLMProvider};
pub use factory::create_provider;

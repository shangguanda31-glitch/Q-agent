pub struct Config {
    pub napcat_ws_url: String,
    pub napcat_http_url: String,
    pub napcat_token: String,
    pub llm_url: String,
    pub embed_url: String,
    pub llm_model: String,
    pub web_port: u16,
    pub llama_server_path: String,
    pub llama_model_path: String,
    pub embed_model_path: String,
    pub llama_gpu_layers: u32,
    pub max_tool_iterations: usize,
    pub claude_code_enabled: bool,
    pub claude_code_timeout_secs: u64,
    pub claude_working_dir: String,
    pub data_dir: String,
}

impl Config {
    pub fn from_env() -> Self {
        let base_dir = std::env::var("PROJECT_ROOT")
            .unwrap_or_else(|_| "PROJECT_ROOT_PLACEHOLDER".to_string());
        let data_dir = std::env::var("DATA_DIR")
            .unwrap_or_else(|_| format!("{}/qq-assistant/data", base_dir));

        Self {
            napcat_ws_url: std::env::var("NAPCAT_WS_URL").unwrap_or_else(|_| "ws://127.0.0.1:4447".to_string()),
            napcat_http_url: std::env::var("NAPCAT_HTTP_URL").unwrap_or_else(|_| "http://127.0.0.1:4444".to_string()),
            napcat_token: std::env::var("NAPCAT_TOKEN").unwrap_or_else(|_| "your_token_here".to_string()),
            llm_url: std::env::var("LLM_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string()),
            embed_url: std::env::var("EMBED_URL").unwrap_or_else(|_| "http://127.0.0.1:8082".to_string()),
            llm_model: std::env::var("LLM_MODEL").unwrap_or_else(|_| "Qwen3.5-9B-Q4_K_M".to_string()),
            web_port: std::env::var("WEB_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(5050),
            llama_server_path: std::env::var("LLAMA_SERVER_PATH").unwrap_or_else(|_| "LLM_DIR_PLACEHOLDERllama-bin/llama-server.exe".to_string()),
            llama_model_path: std::env::var("LLAMA_MODEL_PATH").unwrap_or_else(|_| "LLM_DIR_PLACEHOLDERmodels/Qwen3.5-9B.Q4_K_M.gguf".to_string()),
            embed_model_path: std::env::var("EMBED_MODEL_PATH").unwrap_or_else(|_| "LLM_DIR_PLACEHOLDERmodels/Qwen3.5-0.8B-Q6_K.gguf".to_string()),
            llama_gpu_layers: std::env::var("LLAMA_GPU_LAYERS").ok().and_then(|v| v.parse().ok()).unwrap_or(99),
            max_tool_iterations: std::env::var("MAX_TOOL_ITERATIONS").ok().and_then(|v| v.parse().ok()).unwrap_or(10),
            claude_code_enabled: std::env::var("CLAUDE_CODE_ENABLED").ok().map(|v| v != "0" && v != "false").unwrap_or(true),
            claude_code_timeout_secs: std::env::var("CLAUDE_CODE_TIMEOUT").ok().and_then(|v| v.parse().ok()).unwrap_or(1800),
            claude_working_dir: std::env::var("CLAUDE_WORKING_DIR").unwrap_or_else(|_| format!("{}/qq-assistant/claude_workspace", base_dir)),
            data_dir,
        }
    }
}

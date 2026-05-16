mod agent;
mod config;
mod llm;
mod napcat;
mod notify;
mod store;
mod tools;
mod web;

use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, error, warn};

fn check_llm_server_running(port: &str) -> bool {
    use std::io::{Read, Write};
    let mut stream = match std::net::TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().unwrap(),
        std::time::Duration::from_secs(2),
    ) {
        Ok(s) => s,
        Err(_) => return false,
    };
    // Send a real HTTP request to check if it's an OpenAI-compatible server
    let _ = write!(stream, "GET /v1/models HTTP/1.0\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    let mut resp = String::new();
    let _ = stream.read_to_string(&mut resp);
    let is_llama = resp.contains("200") || resp.contains("object") || resp.contains("model");
    if !is_llama {
        warn!("Port {} is occupied by a non-LLM service (response: {}...), will try another port", port, &resp[..resp.len().min(60)]);
    }
    is_llama
}

fn start_llama_server(cfg: &config::Config) -> (Option<std::process::Child>, String) {
    let ports_to_try = ["8081", "8082", "8083", "8080"];
    let model_path = std::path::Path::new(&cfg.llama_model_path);
    let server_path = &cfg.llama_server_path;

    // First check if any port already has llama-server
    for &port in &ports_to_try {
        if check_llm_server_running(port) {
            info!("LLM server already running on port {}", port);
            return (None, format!("http://127.0.0.1:{}", port));
        }
    }

    if !model_path.exists() {
        warn!("Model file not found: {}. Start llama-server manually.", cfg.llama_model_path);
        return (None, cfg.llm_url.clone());
    }
    if !std::path::Path::new(server_path).exists() {
        warn!("llama-server not found at: {}", server_path);
        return (None, cfg.llm_url.clone());
    }

    for &port in &ports_to_try {
        let addr = format!("127.0.0.1:{}", port);
        if std::net::TcpStream::connect_timeout(&addr.parse().unwrap(), std::time::Duration::from_millis(200)).is_ok() {
            continue; // port in use by something
        }
        warn!("Starting llama-server on port {}...", port);
        let gpu_layers = cfg.llama_gpu_layers.min(40); // 8GB VRAM safe limit
        match std::process::Command::new(server_path)
            .args(["-m", &cfg.llama_model_path, "--host", "127.0.0.1", "--port", port,
                   "-ngl", &gpu_layers.to_string(), "--ctx-size", "8192", "--embeddings", "--pooling", "mean"])
            .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(child) => {
                info!("llama-server started PID {} on port {}", child.id(), port);
                std::thread::sleep(std::time::Duration::from_secs(10));
                return (Some(child), format!("http://127.0.0.1:{}", port));
            }
            Err(e) => error!("Failed to start llama-server on port {}: {}", port, e),
        }
    }

    error!("Could not start llama-server on any port");
    (None, cfg.llm_url.clone())
}

fn build_tool_registry(
    napcat_api: Arc<napcat::api::NapCatApi>,
    schedule_store: Arc<store::ScheduleStore>,
    memory_store: Arc<store::MemoryStore>,
    note_store: Arc<store::NoteStore>,
    llm: Arc<llm::LLMClient>,
    cfg: &config::Config,
) -> Arc<tools::traits::ToolRegistry> {
    let mut reg = tools::traits::ToolRegistry::new();
    reg.register(tools::notify::NotifyTool::new());
    reg.register(tools::qq_read::QQReadTool::new(napcat_api));
    reg.register(tools::schedule::ScheduleTool::new(schedule_store.clone()));
    reg.register(tools::schedule::ScheduleListTool::new(schedule_store));
    reg.register(tools::memory::RememberTool::new(memory_store.clone(), llm.clone()));
    reg.register(tools::memory::RecallTool::new(memory_store, llm));
    reg.register(tools::note_take::NoteTakeTool::new(note_store));
    reg.register(tools::ocr::OcrTool::new());
    if cfg.claude_code_enabled {
        let _ = std::fs::create_dir_all(&cfg.claude_working_dir);
        reg.register(tools::claude_code::ClaudeCodeTool::new(
            cfg.claude_code_timeout_secs, 8192, &cfg.claude_working_dir,
        ));
        info!("Claude Code tool registered");
    }
    Arc::new(reg)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present (not tracked by git)
    let _ = dotenvy::dotenv();

    // Disable proxy for local requests
    unsafe { std::env::set_var("NO_PROXY", "127.0.0.1,localhost"); }
    unsafe { std::env::set_var("no_proxy", "127.0.0.1,localhost"); }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "qq_assistant=info".into()))
        .init();

            let cfg = config::Config::from_env();

    let ver = env!("CARGO_PKG_VERSION");
    println!();
    print!("[36m  ┌───────────────────────────────────────────────────────────┐
");
    print!("[36m  │                                                           │
");
    print!("[36m  │     ███████   █████   ██████  ███████ ███    ██ ████████  │
");
    print!("[36m  │    ██     ██ ██   ██ ██       ██      ████   ██    ██     │
");
    print!("[36m  │    ██  ██ ██ ███████ ██   ███ █████   ██ ██  ██    ██     │
");
    print!("[36m  │    ██   ████ ██   ██ ██    ██ ██      ██  ██ ██    ██     │
");
    print!("[36m  │     ██████   ██   ██  ██████  ███████ ██   ████    ██     │
");
    print!("[36m  │                                                           │
");
    print!("[36m  │ [33m                         v0.1.0                           [36m│
");
    print!("[36m  │                                                           │
");
    print!("[36m  ├───────────────────────────────────────────────────────────┤
");
    print!("[36m  │  [0mNapCat  {}                              [36m│
", cfg.napcat_ws_url);
    print!("[36m  │  [0mLLM     {} (9B, :8081)                    [36m│
", cfg.llm_model);
    print!("[36m  │  [0mEmbed   Qwen3.5-0.8B (0.8B, :8082, CPU)                  [36m│
");
    print!("[36m  │  [0mWeb     http://127.0.0.1:{}                            [36m│
", cfg.web_port);
    print!("[36m  │  [0mData    ./data                                           [36m│
");
    print!("[36m  └───────────────────────────────────────────────────────────┘
");
    print!("[0m
");

    let (_llm_process, llm_url) = start_llama_server(&cfg);
    // Start embed server (0.8B model)
    let embed_port = "8082";
    if !std::net::TcpStream::connect("127.0.0.1:8082").is_ok() {
        let embed_path = std::path::Path::new(&cfg.embed_model_path);
        if embed_path.exists() {
            warn!("Starting embed server on port {}...", embed_port);
            std::process::Command::new(&cfg.llama_server_path)
                .args(["-m", &cfg.embed_model_path, "--host", "127.0.0.1", "--port", embed_port,
                       "-ngl", "0", "--ctx-size", "512", "--embeddings", "--pooling", "mean"])
                .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
                .spawn().ok();
            std::thread::sleep(std::time::Duration::from_secs(5));
        } else {
            warn!("Embed model not found at {}, using main LLM for embeddings", cfg.embed_model_path);
        }
    }

    // Ensure data directory
    let _ = std::fs::create_dir_all(&cfg.data_dir);

    // Channels
    let (raw_tx, _) = broadcast::channel::<napcat::types::OneBotEvent>(1024);
    let (processed_tx, _) = broadcast::channel::<napcat::types::ProcessedEvent>(2048);
    let raw_tx = Arc::new(raw_tx);
    let processed_tx = Arc::new(processed_tx);

    // Stores
    let event_store = Arc::new(store::EventStore::new(&cfg.data_dir));
    let schedule_store = Arc::new(store::ScheduleStore::new(&cfg.data_dir));
    let memory_store = Arc::new(store::MemoryStore::new(&cfg.data_dir));
    let note_store = Arc::new(store::NoteStore::new(&cfg.data_dir));
    let chat_history = Arc::new(store::MessageHistoryStore::new(50));

    // API clients
    let napcat_api = Arc::new(napcat::api::NapCatApi::new(&cfg.napcat_http_url, &cfg.napcat_token));
    let llm = Arc::new(llm::LLMClient::new(&llm_url, &cfg.embed_url, &cfg.llm_model));

    // Tool registry
    let tools = build_tool_registry(napcat_api.clone(), schedule_store.clone(), memory_store.clone(), note_store.clone(), llm.clone(), &cfg);

    // === Spawn WebSocket listener ===
    let ws_tx = raw_tx.clone();
    let ws_url = cfg.napcat_ws_url.clone();
    let ws_token = cfg.napcat_token.clone();
    tokio::spawn(async move {
        loop {
            info!("Connecting to NapCatQQ WebSocket...");
            if let Err(e) = napcat::ws::connect(&ws_url, &ws_token, ws_tx.clone()).await {
                error!("WebSocket disconnected: {}. Reconnecting in 5s...", e);
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    // === Spawn Agent Loop ===
    let agent_rx = raw_tx.subscribe();
    let agent_llm = llm.clone();
    let agent_api = napcat_api.clone();
    let agent_tx = processed_tx.clone();
    let agent_store = event_store.clone();
    let agent_sched = schedule_store.clone();
    let agent_tools = tools.clone();
    let agent_max = cfg.max_tool_iterations;
    let agent_ch = chat_history.clone();
    let agent_mem = memory_store.clone();
    tokio::spawn(async move {
        agent::run(agent_rx, agent_llm, agent_api, (*agent_tx).clone(), agent_store,
                   agent_sched, agent_tools, agent_max, agent_ch, agent_mem).await;
    });

    // === Spawn Web Server ===
    let web_tx = processed_tx.clone();
    let web_store = event_store.clone();
    let web_sched = schedule_store.clone();
    let web_notes = note_store.clone();
    let web_memories = memory_store.clone();
    let web_port = cfg.web_port;
    tokio::spawn(async move {
        let app = web::router(web_tx, web_store, web_sched, web_notes, web_memories);
        let ports = [web_port, 5051, 5052, 5053];
        for &port in &ports {
            match tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await {
                Ok(listener) => {
                    info!("Web panel: http://127.0.0.1:{}/", port);
                    axum::serve(listener, app).await.unwrap();
                    return;
                }
                Err(_) => continue,
            }
        }
        error!("Could not bind any port in {:?}", ports);
    });

    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    Ok(())
}

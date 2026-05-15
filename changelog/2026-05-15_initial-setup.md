# 初始项目搭建

**日期**: 2026-05-15
**类型**: 项目初始化

## 变更内容
- 创建 Rust 项目 `qq-assistant`，使用 `cargo new`
- 添加核心依赖：tokio, reqwest, axum, serde, tungstenite 等
- 实现 NapCatQQ WebSocket 客户端 (napcat/ws.rs)
- 实现 NapCat HTTP API 客户端 (napcat/api.rs)
- 定义 OneBot v11 事件类型 (napcat/types.rs)
- 实现 LLM 客户端 (llm.rs)
- 实现 Windows 通知 (notify.rs)
- 实现 Web 面板 (web/mod.rs + web/static/index.html)
- 实现消息处理流水线 (processor.rs)

## 配置
- NapCatQQ WebSocket: ws://127.0.0.1:4447
- NapCat HTTP API: http://127.0.0.1:4444
- Token: NAPCAT_TOKEN_PLACEHOLDER
- LLM: Qwen3.5-9B on port 8080
- Web 面板: http://127.0.0.1:5050

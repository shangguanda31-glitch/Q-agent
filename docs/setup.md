# 安装与配置指南

## 前置依赖

| 组件 | 版本要求 | 获取方式 |
|------|---------|---------|
| Rust | 1.85+ | rustup.rs |
| NapCatQQ | 最新 | 需提前运行 |
| llama-server | 内置 | 自动启动 |
| Tesseract OCR | 5.x | 需手动安装 |
| Claude Code CLI | 最新 | `npm i -g @anthropic-ai/claude-code` |

## 快速启动

```bash
# 1. 启动 NapCatQQ（确保 WebSocket :4447 + HTTP :4444 已配置）

# 2. 确保 Qwen3.5-9B Q4_K_M GGUF 模型已下载

# 3. 编译并运行
cd qq-assistant
cargo run --release

# 4. 打开面板
# http://127.0.0.1:5050
```

## 配置（环境变量）

| 变量 | 默认值 | 说明 |
|------|--------|------|
| NAPCAT_WS_URL | ws://127.0.0.1:4447 | NapCatQQ WebSocket |
| NAPCAT_HTTP_URL | http://127.0.0.1:4444 | NapCatQQ HTTP API |
| NAPCAT_TOKEN | NAPCAT_TOKEN_PLACEHOLDER | 鉴权 Token |
| WEB_PORT | 5050 | 面板端口（自动回退 5051-5053）|
| LLM_URL | http://127.0.0.1:8080 | llama-server |
| MAX_TOOL_ITERATIONS | 10 | Agent 循环最大轮次 |
| CLAUDE_CODE_TIMEOUT | 120 | Claude Code 超时（秒）|

## 安装 Tesseract OCR

1. 下载安装包：https://github.com/UB-Mannheim/tesseract/releases/latest
2. 安装时勾选中文语言包
3. 或使用自带安装包：`tesseract/tesseract_setup.exe`

## 数据目录

所有持久化数据保存在 `data/` 目录下：

```
data/
├── message_history.json   消息历史
├── schedules.json         日程
├── memories.json          记忆
├── notes.json             笔记
└── message_history.json   消息记录
```

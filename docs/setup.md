# 安装与配置指南

## 前置依赖

| 组件 | 版本要求 | 获取方式 |
|------|---------|---------|
| Rust | 1.85+ | rustup.rs |
| NapCatQQ | 最新 | 需提前运行 |
| llama-server | 内置 | 自动启动 |
| Tesseract OCR | 5.x | 可选，需手动安装 |
| Claude Code CLI | 最新 | `npm i -g @anthropic-ai/claude-code`（可选）|

## 快速启动

```bash
# 1. 启动 NapCatQQ（WebSocket :4447 + HTTP :4444）

# 2. 编译运行
cd qq-assistant
cargo run --release

# 3. 打开面板
# http://127.0.0.1:5050
```

## 配置

### .env 文件（推荐）
复制 `.env.example` 为 `.env`，填入配置。`.env` 不会被 git 追踪。

### 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `NAPCAT_WS_URL` | `ws://127.0.0.1:4447` | NapCatQQ WebSocket |
| `NAPCAT_TOKEN` | — | 鉴权 Token（必须设置）|
| `WEB_PORT` | `5050` | 面板端口 |
| `MAX_TOOL_ITERATIONS` | `10` | Agent 循环最大轮次 |
| `CLAUDE_CODE_TIMEOUT` | `120` | Claude Code 超时（秒）|

## 存储

- 所有数据统一存储在 `data/data.db`（SQLite + WAL 模式）
- 旧 JSON 文件迁移后自动备份为 `.bak`
- 无需手动管理

## 安装 Tesseract OCR（可选）

1. 下载 https://github.com/UB-Mannheim/tesseract/releases/latest
2. 安装时勾选中文语言包
3. 或运行 `tesseract/tesseract_setup.exe`

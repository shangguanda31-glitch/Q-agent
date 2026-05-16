# QAgent · QQ 智慧助理

[English](#english) | [中文](#中文)

---

## 中文

```
┌───────────────────────────────────────────────────────────┐
│                                                           │
│     ███████   █████   ██████  ███████ ███    ██ ████████ │
│    ██     ██ ██   ██ ██       ██      ████   ██    ██    │
│    ██  ██ ██ ███████ ██   ███ █████   ██ ██  ██    ██    │
│    ██   ████ ██   ██ ██    ██ ██      ██  ██ ██    ██    │
│     ██████   ██   ██  ██████  ███████ ██   ████    ██    │
│                                                           │
│                       v0.1.0                              │
│                                                           │
├───────────────────────────────────────────────────────────┤
│  NapCat  ws://127.0.0.1:4447                              │
│  LLM     Qwen3.5-9B-Q4_K_M (9B, :8081)                    │
│  Embed   Qwen3.5-0.8B (0.8B, :8082, CPU)                        │
│  Web     http://127.0.0.1:5050                            │
│  Data    ./data                                           │
└───────────────────────────────────────────────────────────┘
```

QAgent 是一个 Rust 编写的智能 QQ 助理后台服务，通过 NapCatQQ 监听 QQ 消息，利用本地 LLM（Qwen3.5-9B）进行智能分析，通过 Agent 循环自主调用工具处理任务。

### 特性

- **🤖 Agent 循环** — 消息→LLM 判断→调工具→结果回传→继续或完成
- **🧠 三层记忆** — 会话上下文 + SQLite 持久化 + 读写工具
- **🔧 9 个工具** — 通知 / 日程 / 笔记 / 记忆 / OCR / 代码执行 / QQ 读取
- **🖼️ OCR 识别** — 中英文图片文字识别（Tesseract）
- **💻 Claude Code 集成** — 复杂任务自动调用 Claude Code CLI
- **📊 Web 面板** — 会话 / 日程 / 笔记 / 记忆 / 文件实时查看
- **💾 SQLite 存储** — WAL 模式，读写并发
- **🎨 彩色终端** — ANSI 彩色启动界面

### 架构

```
┌──────────┐   ┌──────────────┐   ┌─────────────────────┐
│ NapCatQQ │──▶│  Agent Loop  │──▶│  9 Tools            │
│ WebSocket│   │ (最多 10 轮)  │   │                     │
│ :4447    │   │ LLM 自主判断  │   │ notify/schedule/    │
└──────────┘   │ 调哪个工具    │   │ claude/ocr/note/    │
               │ 上下文摘要    │   │ remember/recall/    │
┌──────────┐   │ 语义记忆     │   │ qq_read            │
│  Web UI  │   └──────────────┘   └─────────────────────┘
│ :5050    │
└──────────┘
  LLM: 8081 (Qwen3.5-9B, 40 GPU layers)
  Embed: 8082 (Qwen3.5-0.8B, CPU)
  存储: data.db (SQLite + WAL)
```

### 快速开始

```bash
# 1. 运行 NapCatQQ（WebSocket :4447 + HTTP :4444，配置 Token）
# 2. 编译运行
cd qq-assistant
cargo run --release
# 3. 打开 http://127.0.0.1:5050
```

### 配置

| 环境变量 | 默认值 | 说明 |
|---------|--------|------|
| `NAPCAT_WS_URL` | `ws://127.0.0.1:4447` | NapCatQQ WebSocket |
| `NAPCAT_TOKEN` | — | 鉴权 Token（必须设置）|
| `WEB_PORT` | `5050` | 面板端口 |

### 项目结构

```
src/
├── agent/      Agent 循环 + 提示词 + 工具调度
├── tools/      9 个工具实现
├── napcat/     NapCatQQ 通信层
├── web/        Web 面板 (Axum + SSE)
├── llm.rs      LLM + Embedding 客户端
├── store.rs    SQLite 持久化
└── notify.rs   Windows 通知

docs/           技术文档
changelog/      修改记录（每次提交必填）
CLAUDE.md       项目规范
```

### 许可

MIT License

---

<span id="english"></span>

## English

```
┌───────────────────────────────────────────────────────────┐
│                                                           │
│     ███████   █████   ██████  ███████ ███    ██ ████████ │
│    ██     ██ ██   ██ ██       ██      ████   ██    ██    │
│    ██  ██ ██ ███████ ██   ███ █████   ██ ██  ██    ██    │
│    ██   ████ ██   ██ ██    ██ ██      ██  ██ ██    ██    │
│     ██████   ██   ██  ██████  ███████ ██   ████    ██    │
│                                                           │
│                       v0.1.0                              │
│                                                           │
├───────────────────────────────────────────────────────────┤
│  NapCat  ws://127.0.0.1:4447                              │
│  LLM     Qwen3.5-9B-Q4_K_M (9B, :8081)                    │
│  Embed   Qwen3.5-0.8B (0.8B, :8082, CPU)                        │
│  Web     http://127.0.0.1:5050                            │
│  Data    ./data                                           │
└───────────────────────────────────────────────────────────┘
```

QAgent is a Rust-based intelligent QQ assistant. It listens to QQ messages via NapCatQQ WebSocket, processes them with a local LLM (Qwen3.5-9B), and autonomously calls tools through an agent loop.

### Features

- **🤖 Agent Loop** — Message → LLM decides → Call tools → Feed back → Continue
- **🧠 3-Layer Memory** — Session context + SQLite persistence + Read/Write tools
- **🔧 9 Tools** — Notify / Schedule / Notes / Memory / OCR / Claude Code / QQ Read
- **🖼️ OCR** — Chinese & English image text recognition (Tesseract)
- **💻 Claude Code** — Complex tasks delegated to Claude Code CLI
- **📊 Web Dashboard** — Conversations / Schedules / Notes / Memories / Files
- **💾 SQLite** — WAL mode, concurrent reads and writes
- **🎨 Colorful Terminal** — ANSI-colored startup banner

### Architecture

```
┌──────────┐   ┌──────────────┐   ┌─────────────────────┐
│ NapCatQQ │──▶│  Agent Loop  │──▶│  9 Tools            │
│ WebSocket│   │ (max 10 iters)│   │                     │
│ :4447    │   │ LLM decides   │   │ notify/schedule/    │
└──────────┘   │ which tool    │   │ claude/ocr/note/    │
               │ context sum.  │   │ remember/recall/    │
┌──────────┐   │ memory search │   │ qq_read            │
│  Web UI  │   └──────────────┘   └─────────────────────┘
│ :5050    │
└──────────┘
  LLM: 8081 (Qwen3.5-9B, 40 GPU layers)
  Embed: 8082 (Qwen3.5-0.8B, CPU)
  Storage: data.db (SQLite + WAL)
```

### Quick Start

```bash
# 1. Run NapCatQQ (WebSocket :4447 + HTTP :4444, configure token)
# 2. Build & run
cd qq-assistant
cargo run --release
# 3. Open http://127.0.0.1:5050
```

### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `NAPCAT_WS_URL` | `ws://127.0.0.1:4447` | NapCatQQ WebSocket |
| `NAPCAT_TOKEN` | — | Auth token（required）|
| `WEB_PORT` | `5050` | Dashboard port |

### Project Structure

```
src/
├── agent/      Agent loop + prompts + tool dispatch
├── tools/      9 tool implementations
├── napcat/     NapCatQQ communication layer
├── web/        Web dashboard (Axum + SSE)
├── llm.rs      LLM + Embedding client
├── store.rs    SQLite persistence
└── notify.rs   Windows notifications

docs/           Technical documentation
changelog/      Change history (required per commit)
CLAUDE.md       Project conventions
```

### License

MIT License

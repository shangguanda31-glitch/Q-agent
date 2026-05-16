# QAgent · QQ 智慧助理

<div align="center">

[**中文**](#lang-cn) · [English](#lang-en)

</div>

---

<div id="lang-cn">

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
│  Embed   Qwen3.5-0.8B (0.8B, :8082, CPU)                  │
│  Web     http://127.0.0.1:5050                            │
│  Data    ./data                                           │
│  Issues  github.com/OrinVoss/Q-agent/issues               │
└───────────────────────────────────────────────────────────┘
```

QAgent 是一个 Rust 编写的智能 QQ 助理后台服务。通过 NapCatQQ 监听 QQ 消息，利用本地 LLM（Qwen3.5-9B）进行智能分析，通过 Agent 循环自主调用工具处理任务。

## 特性

- **🤖 Agent 循环** — 消息→LLM 判断→调工具→结果回传→继续或完成，最多 10 轮
- **🧠 三层记忆** — 会话上下文（20条）+ SQLite 持久化 + 语义向量搜索
- **🔧 10 个工具** — 通知 / 日程 CRUD / 笔记 / 记忆读写 / OCR / Claude Code / QQ 读取
- **🔇 排除列表** — Web 面板管理被忽略的群/用户，不占用 LLM 上下文
- **🖼️ OCR 识别** — 中英文图片文字识别（Tesseract），自动触发
- **💻 Claude Code 集成** — 复杂任务委托给 Claude Code，stream-json 实时进度通知
- **📊 Web 面板** — 会话 / 日程 / 笔记 / 记忆 / 文件 / 排除管理
- **💾 SQLite 存储** — WAL 模式，读写并发，自动 JSON 迁移
- **⏱️ 中文时间解析** — 支持「下周三下午5点」「后天晚上8点半」「5月20号」等自然语言
- **🎨 彩色终端** — ANSI 彩色启动界面

## 架构

```
┌──────────┐   ┌──────────────┐   ┌─────────────────────┐
│ NapCatQQ │──▶│  Agent Loop  │──▶│  10 Tools            │
│ WebSocket│   │ (最多 10 轮)  │   │ notify/schedule/    │
│ :4447    │   │ LLM 自主判断  │   │ claude/ocr/note/    │
└──────────┘   │ 上下文压缩    │   │ remember/recall/    │
               │ 语义记忆注入  │   │ qq_read            │
┌──────────┐   │ 排除列表过滤  │   └─────────────────────┘
│  Web UI  │   └──────────────┘
│ :5050    │         ↓
│ 排除管理  │   ┌──────────────┐
└──────────┘   │ ExclusionStore│
               │ (SQLite)      │
               └──────────────┘
  LLM: 8081 (Qwen3.5-9B, 42-48 t/s, 40 GPU layers)
  Embed: 8082 (Qwen3.5-0.8B, CPU, 1024d vectors)
  存储: data.db (SQLite + WAL)
  Claude Code: claude_workspace/ (讯飞星火 API)
```

## 工具清单

| 工具 | 功能 |
|------|------|
| `notify_user` | Windows 桌面通知弹窗 |
| `schedule_create` | 保存日程（中文时间解析） |
| `schedule_list` | 查看所有日程 |
| `schedule_update` | 更新日程信息（按 ID 精确匹配） |
| `claude_code` | 委托 Claude Code 执行复杂任务 |
| `ocr_image` | Tesseract 中英文 OCR |
| `note_take` | 记录对话笔记 |
| `remember` | 语义记忆写入（含 embedding） |
| `recall` | 语义记忆检索（余弦相似度） |
| `qq_read` | 读取 QQ 群公告等信息 |

## 快速开始

```bash
# 1. 配置环境变量
export NAPCAT_TOKEN="your_token_here"
export LLAMA_MODEL_PATH="models/qwen3.5-9b-q4_k_m.gguf"

# 2. 运行 NapCatQQ（WebSocket :4447 + HTTP :4444）
# 3. 编译运行
cd qq-assistant
cargo run --release

# 4. 打开 http://127.0.0.1:5050
```

## 配置

| 环境变量 | 默认值 | 说明 |
|---------|--------|------|
| `NAPCAT_WS_URL` | `ws://127.0.0.1:4447` | NapCatQQ WebSocket |
| `NAPCAT_HTTP_URL` | `http://127.0.0.1:4444` | NapCatQQ HTTP API |
| `NAPCAT_TOKEN` | `your_token_here` | 鉴权 Token（必须设置实际值）|
| `WEB_PORT` | `5050` | Web 面板端口 |
| `DATA_DIR` | `data` | 数据目录（SQLite 存储位置）|
| `LLM_URL` | `http://127.0.0.1:8081` | LLM API 地址 |
| `EMBED_URL` | `http://127.0.0.1:8082` | Embedding API 地址 |
| `LLM_MODEL` | `qwen3.5-9b` | 模型名称（传给 API）|
| `LLAMA_SERVER_PATH` | `llama-server` | llama-server 路径 |
| `LLAMA_MODEL_PATH` | `models/qwen3.5-9b-q4_k_m.gguf` | LLM 模型文件路径 |
| `EMBED_MODEL_PATH` | `models/qwen3.5-0.8b-q6_k.gguf` | Embedding 模型文件路径 |
| `LLAMA_GPU_LAYERS` | `99` | GPU 层数（笔记本 5070 建议 40）|
| `MAX_TOOL_ITERATIONS` | `10` | Agent 循环最大轮数 |
| `CLAUDE_CODE_ENABLED` | `true` | 是否启用 Claude Code |
| `CLAUDE_CODE_TIMEOUT` | `1800` | Claude Code 超时（秒）|
| `CLAUDE_WORKING_DIR` | `claude_workspace` | Claude Code 工作目录 |
| `TESSERACT_PATH` | — | Tesseract 可执行文件路径 |
| `TESSDATA_PREFIX` | `tesseract/tessdata` | Tesseract 语言数据目录 |

## 项目结构

```
src/
├── agent/      Agent 循环 + 提示词 + 工具调度
│   ├── mod.rs        核心循环编排
│   ├── prompt.rs     System prompt 构建
│   └── dispatcher.rs tool_call 解析
├── tools/      10 个工具实现
│   ├── traits.rs     Tool trait + Registry
│   ├── notify.rs     通知
│   ├── schedule.rs   日程 CRUD
│   ├── claude_code.rs Claude Code 子进程
│   ├── ocr.rs        Tesseract OCR
│   ├── memory.rs     记忆读写
│   ├── note_take.rs  笔记
│   └── qq_read.rs    QQ 信息读取
├── napcat/     NapCatQQ 通信层
├── web/        Web 面板 (Axum + SSE + SPA)
├── llm.rs      LLM + Embedding 客户端
├── store.rs    SQLite 持久化（六表）
└── notify.rs   Windows Toast 通知

docs/           技术文档 + 远景规划 + 完善计划
changelog/      修改记录（每次提交必填）
CLAUDE.md       项目规范与已知教训
```

## 许可

MIT License

---

<div align="center">
<a href="https://github.com/OrinVoss/Q-agent/issues">GitHub Issues</a> ·
<a href="docs/project/polish-plan.md">完善计划</a> ·
<a href="docs/project/vision.md">远景规划</a>
</div>

</div>

---

## English

<div id="lang-en">

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
│  Embed   Qwen3.5-0.8B (0.8B, :8082, CPU)                  │
│  Web     http://127.0.0.1:5050                            │
│  Data    ./data                                           │
│  Issues  github.com/OrinVoss/Q-agent/issues               │
└───────────────────────────────────────────────────────────┘
```

QAgent is a Rust-based intelligent QQ assistant. It listens to QQ messages via NapCatQQ WebSocket, processes them with a local LLM (Qwen3.5-9B), and autonomously calls tools through an agent loop.

## Features

- **🤖 Agent Loop** — Message → LLM decides → Call tools → Feed back → Continue (max 10 iters)
- **🧠 3-Layer Memory** — Session context (20 msgs) + SQLite persistence + Semantic vector search
- **🔧 10 Tools** — Notify / Schedule CRUD / Notes / Memory R/W / OCR / Claude Code / QQ Read
- **🔇 Exclusion List** — Ignore noisy groups/users via Web panel toggle
- **🖼️ OCR** — Chinese & English image text recognition (Tesseract)
- **💻 Claude Code** — Complex tasks via Claude Code CLI with real-time stream-json progress
- **📊 Web Dashboard** — Conversations / Schedules / Notes / Memories / Files / Exclusions
- **💾 SQLite** — WAL mode, concurrent reads/writes, auto JSON migration
- **⏱️ Chinese Time Parsing** — Natural language dates: "next Wed 5pm", "day after tomorrow 8:30pm"
- **🎨 Colorful Terminal** — ANSI-colored startup banner

## Architecture

```
┌──────────┐   ┌──────────────┐   ┌─────────────────────┐
│ NapCatQQ │──▶│  Agent Loop  │──▶│  10 Tools            │
│ WebSocket│   │ (max 10 iters)│   │ notify/schedule/    │
│ :4447    │   │ LLM decides   │   │ claude/ocr/note/    │
└──────────┘   │ context sum.  │   │ remember/recall/    │
               │ memory inject │   │ qq_read            │
┌──────────┐   │ exclude filter│   └─────────────────────┘
│  Web UI  │   └──────────────┘
│ :5050    │         ↓
│exclusions│   ┌──────────────┐
└──────────┘   │ExclusionStore│
               │ (SQLite)     │
               └──────────────┘
  LLM: 8081 (Qwen3.5-9B, 42-48 t/s, 40 GPU layers)
  Embed: 8082 (Qwen3.5-0.8B, CPU, 1024d vectors)
  Storage: data.db (SQLite + WAL)
  Claude Code: claude_workspace/ (Xunfei Spark API)
```

## Tools

| Tool | Description |
|------|-------------|
| `notify_user` | Windows Toast notification |
| `schedule_create` | Create schedule (Chinese time parsing) |
| `schedule_list` | List all schedules |
| `schedule_update` | Update schedule by ID |
| `claude_code` | Execute complex tasks via Claude Code CLI |
| `ocr_image` | Tesseract Chinese/English OCR |
| `note_take` | Record conversation notes |
| `remember` | Semantic memory write (with embedding) |
| `recall` | Semantic memory search (cosine similarity) |
| `qq_read` | Read QQ group notices etc. |

## Quick Start

```bash
# 1. Set environment variables
export NAPCAT_TOKEN="your_token_here"
export LLAMA_MODEL_PATH="models/qwen3.5-9b-q4_k_m.gguf"

# 2. Run NapCatQQ (WebSocket :4447 + HTTP :4444)
# 3. Build & run
cd qq-assistant
cargo run --release

# 4. Open http://127.0.0.1:5050
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `NAPCAT_WS_URL` | `ws://127.0.0.1:4447` | NapCatQQ WebSocket |
| `NAPCAT_HTTP_URL` | `http://127.0.0.1:4444` | NapCatQQ HTTP API |
| `NAPCAT_TOKEN` | — | Auth token (required) |
| `WEB_PORT` | `5050` | Web panel port |
| `LLM_URL` | `http://127.0.0.1:8081` | LLM API address |
| `LLAMA_MODEL_PATH` | `models/qwen3.5...gguf` | Model file path |
| `LLAMA_SERVER_PATH` | `llama-server` | llama-server path |
| `CLAUDE_CODE_TIMEOUT` | `1800` | Claude Code timeout (s) |
| `TESSERACT_PATH` | — | Tesseract executable path |
| `TESSDATA_PREFIX` | `tesseract/tessdata` | Tesseract language data |

## Project Structure

```
src/
├── agent/      Agent loop + prompts + tool dispatch
├── tools/      10 tool implementations
├── napcat/     NapCatQQ communication layer
├── web/        Web dashboard (Axum + SSE + SPA)
├── llm.rs      LLM + Embedding client
├── store.rs    SQLite persistence (6 tables)
└── notify.rs   Windows Toast notifications

docs/           Documentation + vision + polish plan
changelog/      Change history
CLAUDE.md       Project conventions
```

## License

MIT License

---

<div align="center">
<a href="https://github.com/OrinVoss/Q-agent/issues">GitHub Issues</a> ·
<a href="docs/project/polish-plan.md">Polish Plan</a> ·
<a href="docs/project/vision.md">Vision</a>
</div>

</div>

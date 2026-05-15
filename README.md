# Q-Agent · QQ 智慧助理

[English](#english) | [中文](#中文)

---

## 中文

QQ 智慧助理是一个 Rust 编写的智能后台服务，连接 NapCatQQ 监听 QQ 消息，利用本地 LLM（Qwen3.5-9B）进行智能分析，通过 Agent 循环自主调用工具处理任务。

### 特性

- **🤖 Agent 循环** — 收到消息 → LLM 判断 → 调工具 → 结果回传 → 继续或完成
- **🧠 三层记忆** — 会话上下文 + 文件持久化 + 读写工具
- **🔧 8 个工具** — 通知 / 日程 / 笔记 / 记忆 / OCR / 代码执行 / QQ 读取
- **🖼️ OCR 识别** — 中英文图片文字识别（Tesseract）
- **💻 Claude Code 集成** — 复杂任务自动调用 Claude Code CLI
- **📊 Web 面板** — 消息 / 日程 / 笔记 / 记忆 / 文件 实时查看
- **💾 全持久化** — 所有数据存入 JSON 文件，重启不丢

### 快速开始

```bash
# 依赖
# 1. 安装 Rust: https://rustup.rs
# 2. 运行 NapCatQQ（WebSocket :4447 + HTTP :4444）
# 3. 准备 Qwen3.5-9B GGUF 模型

# 编译运行
cd qq-assistant
cargo run --release

# 打开面板
# http://127.0.0.1:5050
```

### 配置

| 环境变量 | 默认值 | 说明 |
|---------|--------|------|
| `NAPCAT_WS_URL` | `ws://127.0.0.1:4447` | NapCatQQ WebSocket |
| `NAPCAT_TOKEN` | `20080103` | 鉴权 Token |
| `WEB_PORT` | `5050` | 面板端口 |
| `CLAUDE_CODE_TIMEOUT` | `120` | Claude Code 超时(秒) |

### 项目结构

```
qq-assistant/
├── src/               Rust 源码
│   ├── agent/         Agent 循环（LLM 编排 + 工具调度）
│   ├── tools/         8 个工具实现
│   ├── napcat/        NapCatQQ 通信层
│   ├── web/           Web 面板（Axum + SSE）
│   ├── llm.rs         LLM 客户端
│   └── store.rs       数据持久化
├── docs/              技术文档
├── changelog/         修改记录
└── CLAUDE.md          项目规范
```

### 许可

MIT License

---

<span id="english"></span>

## English

Q-Agent is an intelligent Rust-based background service that connects to NapCatQQ to monitor QQ messages, uses a local LLM (Qwen3.5-9B) for analysis, and autonomously calls tools through an agent loop to handle tasks.

### Features

- **🤖 Agent Loop** — Receive → LLM decides → Call tools → Feed back → Continue
- **🧠 3-Layer Memory** — Context + File persistence + Read/Write tools
- **🔧 8 Tools** — Notify / Schedule / Notes / Memory / OCR / Code / QQ Read
- **🖼️ OCR** — Chinese & English image text recognition (Tesseract)
- **💻 Claude Code** — Complex tasks delegated to Claude Code CLI
- **📊 Web Dashboard** — Messages / Schedules / Notes / Memory / Files
- **💾 Full Persistence** — All data in JSON files, survives restart

### Quick Start

```bash
# Prerequisites
# 1. Install Rust: https://rustup.rs
# 2. Run NapCatQQ (WebSocket :4447 + HTTP :4444)
# 3. Download Qwen3.5-9B GGUF model

# Build & run
cd qq-assistant
cargo run --release

# Open dashboard
# http://127.0.0.1:5050
```

### Project Structure

```
qq-assistant/
├── src/               Rust source
│   ├── agent/         Agent loop (LLM orchestration + tool dispatch)
│   ├── tools/         8 tool implementations
│   ├── napcat/        NapCatQQ communication
│   ├── web/           Web dashboard (Axum + SSE)
│   ├── llm.rs         LLM client
│   └── store.rs       Data persistence
├── docs/              Documentation
├── changelog/         Change history
└── CLAUDE.md          Project conventions
```

### License

MIT License

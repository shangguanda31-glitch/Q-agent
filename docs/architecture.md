# QQ 智慧助理 — 技术架构文档

## 项目概述

QQ 智慧助理是一个 Rust 编写的后台服务，通过 NapCatQQ WebSocket 监听 QQ 消息，利用本地 LLM（llama.cpp）进行智能分析，并提供工具链自动处理任务。

## 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                       qq-assistant                          │
│                                                             │
│  ┌──────────┐   ┌──────────────┐   ┌─────────────────────┐ │
│  │ NapCatQQ │──▶│  Agent Loop │──▶│   Tools (8)         │ │
│  │ WebSocket│   │              │   │   - notify_user     │ │
│  │ :4447    │   │  LLM decides │   │   - schedule_create │ │
│  └──────────┘   │  which tool  │   │   - claude_code     │ │
│                 │  to call     │   │   - ocr_image       │ │
│  ┌──────────┐   │              │   │   - note_take       │ │
│  │  Web UI  │   │  max 10 iters│   │   - remember/recall │ │
│  │ :5050    │   └──────────────┘   │   - qq_read         │ │
│  └──────────┘                      └─────────────────────┘ │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Stores                                              │   │
│  │  EventStore | ScheduleStore | MemoryStore | NoteStore│   │
│  │  (JSON 文件持久化)                                    │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  LLM: llama-server (Qwen3.5-9B, port 8081)           │   │
│  │  OCR: Tesseract (chi_sim+eng)                        │   │
│  │  Code: Claude Code CLI                               │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## 模块说明

### 核心模块 (src/)

| 模块 | 文件 | 职责 |
|------|------|------|
| `main.rs` | 入口 | 启动各模块、信号处理 |
| `config.rs` | 配置 | 环境变量驱动的配置系统 |
| `llm.rs` | LLM 客户端 | llama.cpp HTTP API 调用 |
| `store.rs` | 存储层 | 消息/日程/记忆/笔记持久化 |
| `notify.rs` | 通知 | Windows Toast 通知 |

### 代理循环 (src/agent/)

| 文件 | 职责 |
|------|------|
| `mod.rs` | AgentLoop 编排：收到消息→LLM→调工具→结果回传→循环 |
| `prompt.rs` | System Prompt 构建（含工具描述+记忆上下文） |
| `dispatcher.rs` | 解析 LLM 响应中的 `<tool_call>` XML 标签 |

### 工具 (src/tools/)

所有工具实现 `Tool` trait，注册在 `ToolRegistry` 中：

| 工具 | 功能 | 参数 |
|------|------|------|
| `notify_user` | Windows 桌面通知 | title, body |
| `schedule_create` | 创建日程 | title, info |
| `schedule_list` | 查看日程 | 无 |
| `claude_code` | 调用 Claude Code CLI 处理任务 | prompt |
| `ocr_image` | Tesseract OCR 识别图片文字 | image_path |
| `note_take` | 记录笔记 | content, speaker, source |
| `remember` | 记忆写入 | content |
| `recall` | 记忆读取 | query |

### 数据持久化 (data/)

| 文件 | 格式 | 内容 |
|------|------|------|
| `message_history.json` | JSON Array | 最近 500 条处理过的消息 |
| `schedules.json` | JSON Array | 日程条目（pending/done） |
| `memories.json` | JSON Array | LLM 主动保存的记忆 |
| `notes.json` | JSON Array | 对话中记录的重要笔记 |
| `notes.json` | JSON Array | 对话中记录的重要笔记 |

## 三层记忆系统

1. **上下文层**: Agent 循环中的 `messages` Vec，每会话 20 条历史
2. **文件层**: `data/memories.json`，持久化存储
3. **工具层**: `remember` / `recall` 工具，LLM 可主动读写

上下文窗口管理：超过 6144 token 时自动裁剪旧消息。

## LLM 配置

- 模型: Qwen3.5-9B (Q4_K_M, GGUF)
- 推理引擎: llama-server (llama.cpp)
- 端口: 8081 (自动检测+端口迁移)
- 上下文: 8192 tokens
- GPU 层数: 40 (RTX 5070 Laptop 8GB VRAM)
- 温度: 0.3 (agent_chat)

## 外部依赖

| 组件 | 用途 | 安装方式 |
|------|------|----------|
| llama-server | LLM 推理 | 内置，自动启动 |
| Tesseract OCR | 图片文字识别 | 需手动安装 |
| Claude Code CLI | 复杂任务执行 | `npm install -g @anthropic-ai/claude-code` |
| NapCatQQ | QQ 协议层 | 需预先运行 |

## 启动流程

1. 加载配置 → 2. 检测/启动 llama-server → 3. 连接 NapCatQQ WebSocket → 4. 启动 Agent 循环 → 5. 启动 Web 面板

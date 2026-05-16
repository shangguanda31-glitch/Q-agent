# QQ 智慧助理 — 技术架构文档

## 项目概述

QQ 智慧助理是一个 Rust 编写的后台服务，通过 NapCatQQ WebSocket 监听 QQ 消息，利用本地 LLM（llama.cpp, Qwen3.5-9B）进行智能分析，通过 Agent 循环自主调用工具处理任务。

## 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                       qq-assistant                          │
│                                                             │
│  ┌──────────┐   ┌──────────────┐   ┌─────────────────────┐ │
│  │ NapCatQQ │──▶│  Agent Loop  │──▶│   10 Tools         │ │
│  │ WebSocket│   │              │   │   notify_user       │ │
│  │ :4447    │   │ LLM 自行判断  │   │   schedule_create   │ │
│  └──────────┘   │ 调哪个工具    │   │   schedule_list     │ │
│                 │ 最多 10 轮   │   │   claude_code       │ │
│  ┌──────────┐   │ 上下文摘要    │   │   ocr_image         │ │
│  │  Web UI  │   │ 语义记忆     │   │   note_take         │ │
│  │ :5050    │   └──────────────┘   │   remember/recall   │ │
│  └──────────┘                      │   qq_read           │ │
│                                    └─────────────────────┘ │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  存储层 (SQLite + WAL)                               │   │
│  │  单文件 data/data.db，WAL 模式支持读写并发            │   │
│  │  表: events / schedules / memories / notes           │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  LLM: llama-server (Qwen3.5-9B)                      │   │
│  │  ➜ 端口 8081                                         │   │
│  │  ➜ --embeddings --pooling mean (4096d 向量)          │   │
│  │  OCR: Tesseract (chi_sim+eng)                        │   │
│  │  Code: Claude Code CLI                               │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## 核心数据流

```
QQ消息 → 消息解析 → 加载会话历史 + 记忆 → LLM 判断
  → 有 <tool_call>? → 执行工具 → 结果回传 → 继续循环
  → 无 → 保存结果 → 推送到 Web 面板
```

## 模块说明

### 核心模块 (src/)

| 模块 | 文件 | 职责 |
|------|------|------|
| `main.rs` | 入口 | 启动各模块、信号处理、dotenv |
| `config.rs` | 配置 | 环境变量驱动的配置系统 |
| `llm.rs` | LLM 客户端 | llama.cpp HTTP API + embeddings |
| `store.rs` | 存储层 | SQLite 持久化（WAL 模式） |
| `notify.rs` | 通知 | Windows Toast 通知 |

### 代理循环 (src/agent/)

| 文件 | 职责 |
|------|------|
| `mod.rs` | AgentLoop 编排、上下文摘要、语义记忆加载 |
| `prompt.rs` | System Prompt 构建（工具描述 + 记忆上下文） |
| `dispatcher.rs` | 解析 LLM 响应中的 `<tool_call>` XML 标签 |

### 工具 (src/tools/)

所有工具实现 `Tool` trait（tools/traits.rs），注册在 `ToolRegistry` 中：

| 工具 | 功能 | 参数 |
|------|------|------|
| `notify_user` | Windows 桌面通知 | title, body |
| `schedule_create` | 创建日程 | title, info |
| `schedule_list` | 查看日程 | 无 |
| `claude_code` | 调用 Claude Code CLI（max-iter 50） | prompt |
| `ocr_image` | Tesseract OCR 中英文 | image_path |
| `note_take` | 记录笔记 | content, speaker, source |
| `remember` | 语义记忆写入 | content |
| `recall` | 语义记忆读取（余弦相似度） | query |
| `qq_read` | 读取群公告等信息 | action, group_id |

## 存储层

- **引擎**: SQLite + WAL 模式
- **路径**: `data/data.db`
- **并发**: parking_lot::Mutex（用户态快速锁）+ WAL 读写并发
- **表结构**:
  - `events` — 消息历史（最多 500 条）
  - `schedules` — 日程（pending/done/reminded）
  - `memories` — 语义记忆（含 embedding 向量）
  - `notes` — 笔记

## 三层记忆系统

1. **上下文层**: Agent 循环 messages Vec，每会话 20 条历史
2. **语义记忆层**: `data.db` 中的 memories 表，每条含 4096d embedding
3. **工具层**: `remember` / `recall` 工具，LLM 主动读写

写入时自动生成 embedding，搜索时余弦相似度排序 + 关键词降级。

## 上下文管理

超 6144 token 时，将旧消息发给 LLM 做摘要压缩后注入上下文开头（保留最近 4 条完整消息）。

## LLM 配置

- 模型: Qwen3.5-9B (Q4_K_M, GGUF)
- 推理引擎: llama-server (llama.cpp)
- 端口: 8081（自动检测，端口被占时回退 8082/8083）
- 上下文: 8192 tokens
- 生成温度: 0.3 (agent_chat)
- Embedding: 同一个模型 + `--embeddings --pooling mean`，4096 维
- GPU 层数: 40（RTX 5070 Laptop 8GB VRAM）
- 模型路径自动检测，无需手动配置

## Web 面板

- 端口: 5050（TIME_WAIT 时自动回退 5051-5053）
- 框架: Axum + SSE + 静态文件
- 标签页: 消息（按会话分组）/ 日程 / 笔记 / 记忆 / 文件

## 外部依赖

| 组件 | 用途 | 安装方式 |
|------|------|----------|
| llama-server | LLM 推理 + embedding | 内置，自动启动 |
| Tesseract OCR | 图片文字识别 | 需手动下载 |
| Claude Code CLI | 复杂任务执行 | `npm i -g @anthropic-ai/claude-code` |
| NapCatQQ | QQ 协议通信层 | 需预先启动 |

## 启动流程

加载配置 (.env) → 检测/启动 llama-server → 连接 NapCatQQ WebSocket → 启动 Agent 循环 → 启动 Web 面板

## 关键路径

| 路径 | 说明 |
|------|------|
| `src/main.rs` | 入口 |
| `src/agent/mod.rs` | AgentLoop 核心 |
| `src/tools/traits.rs` | Tool trait |
| `src/llm.rs` | LLM + Embedding 客户端 |
| `src/store.rs` | SQLite 持久化 |
| `src/web/mod.rs` | Web 面板 API |
| `changelog/` | 修改记录（必须） |

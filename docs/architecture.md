# QQ 智慧助理 — 技术架构文档

## 项目概述

QQ 智慧助理是一个 Rust 编写的后台服务，通过 NapCatQQ WebSocket 监听 QQ 消息，利用本地 LLM（llama.cpp, Qwen3.5-9B）进行智能分析，通过 Agent 循环自主调用工具处理任务。

## 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                       qq-assistant                          │
│                                                             │
│  ┌──────────┐   ┌──────────────┐   ┌─────────────────────┐ │
│  │ NapCatQQ │──▶│  Agent Loop  │──▶│   10 Tools           │ │
│  │ WebSocket│   │              │   │   notify_user       │ │
│  │ :4447    │   │ LLM 自行判断  │   │   schedule_create   │ │
│  └──────────┘   │ 调哪个工具    │   │   claude_code       │ │
│                 │ 最多 10 轮   │   │   ocr_image         │ │
│  ┌──────────┐   │ 上下文摘要    │   │   note_take         │ │
│  │  Web UI  │   │ 语义记忆     │   │   remember/recall   │ │
│  │ :5050    │   └──────────────┘   │   qq_read           │ │
│  └──────────┘                      └─────────────────────┘ │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  存储层 (SQLite + WAL)          data/data.db         │   │
│  │  表: events / schedules / memories / notes           │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  LLM 服务                      │  OCR               │   │
│  │  ┌─────────┐  ┌─────────┐     │  Tesseract         │   │
│  │  │ 8081    │  │ 8082    │     │  (chi_sim+eng)     │   │
│  │  │ 9B 文本 │  │ 0.8B 向量│     │                    │   │
│  │  │ 生成    │  │ 1024d   │     │  Claude Code CLI   │   │
│  │  └─────────┘  └─────────┘     │  (复杂任务)         │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## 核心数据流

```
QQ消息 → 消息解析 → 加载会话历史 + 语义记忆 → LLM 判断
  → 有 <tool_call>? → 执行工具 → 结果回传 → 继续循环
  → 无 → 保存结果 → 推送到 Web 面板
```

## 模块说明

### 核心模块 (src/)

| 模块 | 文件 | 职责 |
|------|------|------|
| `main.rs` | 入口 | 启动各模块、信号处理、dotenv |
| `config.rs` | 配置 | 环境变量驱动的配置系统 |
| `llm.rs` | LLM 客户端 | llama.cpp HTTP API + 独立 embedding |
| `store.rs` | 存储层 | SQLite 持久化（WAL 模式） |
| `notify.rs` | 通知 | Windows Toast 通知 |

### 代理循环 (src/agent/)

| 文件 | 职责 |
|------|------|
| `mod.rs` | AgentLoop 编排、上下文摘要、语义记忆加载 |
| `prompt.rs` | System Prompt 构建（工具描述 + 记忆上下文） |
| `dispatcher.rs` | 解析 `<tool_call>` XML 标签 |

### 工具 (src/tools/)

| 工具 | 功能 |
|------|------|
| `notify_user` | Windows 桌面通知 |
| `schedule_create` | 日程创建（中文时间解析） |
| `schedule_list` | 日程列表 |
| `claude_code` | Claude Code CLI 任务执行 |
| `ocr_image` | 图片文字识别 |
| `note_take` | 笔记记录 |
| `remember` | 语义记忆写入（自动生成 embedding） |
| `recall` | 语义记忆读取（余弦相似度） |
| `qq_read` | 读取群公告 |

## 存储层

- **引擎**: SQLite + WAL 模式
- **路径**: `data/data.db`
- **并发**: parking_lot::Mutex + WAL 读写并发
- **表**: events(500条), schedules, memories(含向量), notes

## 三层记忆系统

1. **上下文层**: 每会话 20 条历史
2. **语义记忆层**: memories 表 + 1024d embedding(0.8B 模型)
3. **工具层**: remember / recall 工具

写入时通过 8082 端口 embed 服务器生成向量，搜索时余弦相似度排序 + 关键词降级。

## 上下文管理

超 6144 token 时，旧消息由 LLM 摘要压缩后注入上下文开头，保留最近 4 条完整消息。

## LLM 配置

| 实例 | 端口 | 模型 | 用途 | GPU 层数 | 上下文 |
|------|------|------|------|---------|-------|
| 主服务 | 8081 | Qwen3.5-9B (Q4_K_M) | 文本生成、Agent | 40层 | 8192 |
| Embed | 8082 | Qwen3.5-0.8B (Q6_K) | 语义向量 | 0(CPU) | 512 |

- Embedding 维度: 1024 (0.8B)
- 自动检测端口占用并回退

## Web 面板

- 端口 5050（TIME_WAIT 时回退 5051-5053）
- Axum + SSE
- 标签页: 消息(会话分组) / 日程 / 笔记 / 记忆 / 文件

## 外部依赖

| 组件 | 用途 |
|------|------|
| llama-server (内置) | LLM 推理 + embedding |
| Tesseract OCR (可选) | 文字识别 |
| Claude Code CLI (可选) | 复杂任务 |
| NapCatQQ | QQ 协议层 |

## 启动流程

加载 .env → 检测/启动 llama-server(8081) → 启动 embed(8082) → 连接 NapCatQQ → Agent 循环 → Web 面板

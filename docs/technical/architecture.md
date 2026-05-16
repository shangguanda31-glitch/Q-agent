# QAgent — 技术架构文档

## 项目概述

QAgent 是一个 Rust 编写的智能助理后台服务。通过 NapCatQQ WebSocket 监听 QQ 消息，利用本地 LLM（llama.cpp, Qwen3.5-9B）进行智能分析，通过 Agent 循环自主调用工具处理任务。只读不写，纯本地运行。

## 系统架构

```
┌──────────┐   ┌──────────────┐   ┌──────────────────────┐
│ NapCatQQ │──▶│  Agent Loop  │──▶│   10 Tools           │
│ WebSocket│   │ (最多 10 轮)  │   │ notify_user         │
│ :4447    │   │              │   │ schedule_create/list │
└──────────┘   │ LLM 自行判断  │   │ schedule_update     │
               │ 调哪个工具    │   │ claude_code         │
┌──────────┐   │ 上下文压缩    │   │ ocr_image           │
│  Web UI  │   │ 语义记忆注入  │   │ note_take           │
│ :5050    │   │ 排除列表过滤  │   │ remember/recall     │
│ 排除管理  │   └──────────────┘   │ qq_read             │
└──────────┘                      └──────────────────────┘
               ┌──────────────────────────────────────────┐
               │  存储层 (SQLite + WAL) data/data.db       │
               │  表: events/schedules/memories/notes/     │
               │       exclusions                         │
               └──────────────────────────────────────────┘
```

## 核心流程

### 1. 消息接收
NapCatQQ WebSocket 接收 OneBot v11 协议消息 → 解析为统一事件 → 送入优先级队列。

优先级规则：@消息 > 长文本 > 短文本（BinaryHeap 排序）。

### 2. 排除过滤
弹出队列后检查 ExclusionStore，被排除的群/用户直接跳过，不占用 LLM 上下文。

### 3. Agent 循环
每条消息进入 `handle_message()`，执行最多 10 轮迭代：

```
构建 prompt → 调 LLM → 解析响应
    ├─ 有 <tool_call> → 执行工具 → 结果回传 → 继续循环
    └─ 无 <tool_call> → 返回最终文本
```

- System prompt 包含工具列表 + 相关记忆（语义检索 top-10）
- 每轮携带前 20 条对话历史
- 超过 6144 token 时 LLM 摘要压缩（保留最后 4 条）
- 中文 token 估算：`byte_len * 2 / 5`（原 `byte_len / 4` 低估一半）
- 支持多工具同一轮调用（一次回复多个 `<tool_call>`）
- 工具调用 JSON 解析失败时记录 warn 日志

### 4. 图片处理流程
```
消息含图片 → 下载到 image_cache/ → 注入 OCR 提示 → LLM 决定是否调 ocr_image
                                                      ↓
                                           Tesseract OCR (chi_sim+eng)
                                                      ↓
                                          结果返回给 LLM
```

- 文件名格式：`img_{user_id}_{timestamp}_{idx}.jpg`
- 缓存目录：`image_cache/`（相对运行目录）

### 5. 三层记忆

| 层级 | 实现 | 说明 |
|------|------|------|
| 上下文 | MessageHistoryStore（内存） | 每 chat_id 保留 20 条最近消息 |
| 持久化 | SQLite（events/memories/notes） | 重启后可通过 recall/note_take 访问 |
| 语义 | Embedding 向量（余弦相似度） | 每次消息自动生成 query embedding 搜索 |

### 6. Claude Code 集成
```
LLM 调 claude_code(prompt) → 子进程 claude -p "..." 
  → --output-format stream-json --include-partial-messages
  → 实时解析 thinking 进度 → 每 10s 通知
  → 结果回传给 LLM 继续判断
```

- 超时 1800s，最多 2 个并发（Semaphore 队列）
- 工作目录 `claude_workspace/`，配置在 `claude.json`
- 外部 API：讯飞星火 `astron-code-latest`

## 工具系统

所有工具实现 `Tool` trait：

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;  // JSON Schema
    async fn execute(&self, args: Value) -> ToolResult;
}
```

注册在 `ToolRegistry` 中，LLM 通过描述和参数 Schema 自主选择调用。

### 工具清单

| 工具 | 功能 | 参数 |
|------|------|------|
| `notify_user` | Windows Toast 通知 | title, body |
| `schedule_create` | 创建日程 | title, time?, info? |
| `schedule_list` | 查看日程 | 无 |
| `schedule_update` | 更新/自动创建日程 | id?, title?, time?, info |
| `claude_code` | Claude Code 复杂任务 | prompt |
| `ocr_image` | Tesseract 中英文 OCR | image_path |
| `note_take` | 记录笔记 | content, speaker, source |
| `remember` | 语义记忆写入（含 embedding） | content |
| `recall` | 语义记忆检索 | query |
| `qq_read` | QQ 群信息读取 | action, group_id? |

## 存储设计

SQLite WAL 模式，5 张表：

| 表 | 用途 | 关键字段 |
|----|------|---------|
| events | 消息事件日志（最多 500 条） | time, message_type, raw_text |
| schedules | 日程 | title, time, time_parsed, description, status |
| memories | 语义记忆（含 1024d embedding） | content, tags, embedding |
| notes | 笔记 | content, speaker, source, group_id |
| exclusions | 排除列表 | exclude_type, target_id |

## 部署架构

```
┌──────────────────────────────────────────────┐
│                  你的电脑                      │
│                                                │
│  ┌────────────┐  ┌────────────────────────┐  │
│  │ QAgent     │  │ llama-server           │  │
│  │ Agent Loop │  │ Qwen3.5-9B  :8081      │  │
│  │ Web :5050  │  │ Qwen3.5-0.8B :8082     │  │
│  │ SQLite     │  │ (embedding, CPU)        │  │
│  └─────┬──────┘  └────────────────────────┘  │
│        │                                      │
│  ┌─────┴──────┐  ┌────────────────────────┐  │
│  │ NapCatQQ   │  │ Tesseract OCR           │  │
│  │ WS :4447   │  │ tessdata/               │  │
│  │ HTTP :4444 │  │ chi_sim+eng             │  │
│  └────────────┘  └────────────────────────┘  │
│                                                │
│  ┌────────────────────────────────────────┐  │
│  │ Claude Code (子进程)                     │  │
│  │ claude_workspace/                       │  │
│  │ 讯飞星火 API (外部)                     │  │
│  └────────────────────────────────────────┘  │
└──────────────────────────────────────────────┘
```

## 配置

完整 18 个环境变量见 `README.md` 配置表，依赖详情见 `docs/technical/dependencies.md`。

**关键路径：**
- 模型文件：`LLAMA_MODEL_PATH` / `EMBED_MODEL_PATH`
- 数据目录：`DATA_DIR`（默认 `data/`）
- 图片缓存：`image_cache/`（运行目录下）

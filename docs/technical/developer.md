# 开发者文档

## 项目概览

QAgent 是一个 Rust 编写的智能 QQ 助理后台服务。架构核心是 Agent 循环：LLM 自主判断消息，调用工具，处理结果。

**技术栈**：Rust + tokio + Axum + SQLite + llama.cpp

---

## 环境搭建

### 前置依赖

| 工具 | 用途 | 获取方式 |
|------|------|---------|
| Rust 1.85+ | 编译运行 | rustup.rs |
| llama.cpp | LLM 推理 | 预编译或自行编译 |
| Qwen3.5-9B Q4_K_M | 主模型 | HuggingFace |
| NapCatQQ | QQ 协议桥接 | GitHub Releases |
| Tesseract 5.x | OCR（可选） | 项目自带 |
| Claude Code CLI | 复杂任务（可选） | `npm i -g @anthropic-ai/claude-code` |

### 快速开始

```bash
# 1. 克隆
git clone https://github.com/OrinVoss/Q-agent.git
cd qq-assistant

# 2. 配置环境变量
cp .env.example .env
# 编辑 .env，填入 NAPCAT_TOKEN 和模型路径

# 3. 启动 NapCatQQ（WebSocket :4447 + HTTP :4444）

# 4. 编译运行
cargo run --release

# 5. 打开 Web 面板
# http://127.0.0.1:5050
```

---

## 项目结构

```
src/
├── main.rs              # 入口：启动 LLM → WebSocket → Agent → Web
├── config.rs            # 环境变量配置（18 个）
├── llm.rs               # LLM + Embedding 客户端
├── store.rs             # SQLite 持久化（6 表）
├── notify.rs            # Windows Toast 通知
│
├── agent/               # Agent 循环核心
│   ├── mod.rs           # 消息处理 + AgentLoop
│   ├── prompt.rs        # System prompt 构建
│   └── dispatcher.rs    # <tool_call> XML 解析
│
├── tools/               # 10 个工具
│   ├── traits.rs        # Tool trait + ToolRegistry
│   ├── notify.rs        # Windows 通知
│   ├── schedule.rs      # 日程 CRUD
│   ├── claude_code.rs   # Claude Code 子进程
│   ├── ocr.rs           # Tesseract OCR
│   ├── memory.rs        # 语义记忆读写
│   ├── note_take.rs     # 笔记记录
│   └── qq_read.rs       # QQ 群信息读取
│
├── napcat/              # NapCatQQ 通信
│   ├── ws.rs            # WebSocket 客户端
│   ├── api.rs           # HTTP API 客户端
│   └── types.rs         # OneBot v11 事件类型
│
└── web/                 # Web 面板
    ├── mod.rs           # Axum 路由 + API 端点
    └── static/
        └── index.html   # 单页应用
```

---

## 核心架构

### Agent 循环流程

```
QQ 消息 → 优先级队列 → 排除过滤
    ↓
构建 system prompt（工具列表 + 相关记忆）
    ↓
调 LLM → 解析响应
    ├─ 有 <tool_call> → 执行工具 → 结果回传 → 继续循环
    └─ 无 <tool_call> → 返回最终文本
```

- 最多 10 轮迭代（`MAX_TOOL_ITERATIONS`）
- 相同输出 3 次中止
- 上下文超过 6144 token 时自动摘要压缩

### 三层记忆

| 层级 | 实现 | 持久化 | 容量 |
|------|------|--------|------|
| 上下文 | MessageHistoryStore | SQLite (`chat_history` 表) | 50 条/会话 |
| 持久化 | SQLite stores | SQLite | 无限制 |
| 语义 | Embedding 向量 | SQLite + 余弦相似度 | 10 条/次召回 |

### 消息处理

```rust
// 每条消息独立处理
handle_message(msg, llm, api, ...).await;

// LLM 调用
let response = llm.agent_chat(&messages, &system_prompt, None).await;

// 工具结果回传
messages.push(AgentMessage {
    role: "user".to_string(),
    content: format!("<tool_result name=\"{}\">\n{}\n</tool_result>", name, result),
});
```

---

## 工具系统

### Tool trait

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;  // JSON Schema
    async fn execute(&self, args: Value) -> ToolResult;
}
```

### 注册工具

在 `main.rs` 的 `build_tool_registry()` 中注册：

```rust
let mut reg = ToolRegistry::new();
reg.register(MyTool::new());
```

工具会自动出现在 LLM 的可用工具列表中。

### ToolResult

```rust
pub struct ToolResult {
    pub success: bool,
    pub output: String,
}

impl ToolResult {
    pub fn ok(msg: impl Into<String>) -> Self { ... }
    pub fn fail(msg: impl Into<String>) -> Self { ... }
}
```

---

## LLM 调用

### agent_chat（主要接口）

```rust
pub async fn agent_chat(
    &self,
    messages: &[AgentMessage],    // 对话历史
    system_prompt: &str,          // 系统提示词
    image_b64: Option<&str>,      // 图片（可选）
) -> anyhow::Result<String>
```

### embed（语义向量）

```rust
pub async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>
```

返回 1024 维向量，用于余弦相似度搜索。

---

## 存储层

### 数据库表

```sql
events       -- 消息日志（上限 500 条）
schedules    -- 日程
memories     -- 语义记忆（含 1024d embedding）
notes        -- 笔记
exclusions   -- 排除列表
chat_history -- 会话历史（持久化）
```

### 添加新表

1. 在 `open_db()` 的 `CREATE TABLE` 中加建表语句
2. 创建对应的 Store 结构体（参考 `ExclusionStore`）
3. 在 `main.rs` 中实例化
4. 在 Web 面板中添加 API 端点（如果需要）

---

## Web API

### 端点一览

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/` | Web 面板首页 |
| GET | `/events` | SSE 实时事件流 |
| GET | `/api/history` | 消息历史 |
| GET | `/api/schedules` | 日程列表 |
| POST | `/api/schedules/done` | 标记日程完成 |
| POST | `/api/schedules/delete` | 删除日程 |
| GET | `/api/notes` | 笔记列表 |
| GET | `/api/memories` | 记忆列表 |
| GET | `/api/workspace` | 工作区文件列表 |
| GET | `/api/workspace/{path}` | 工作区文件内容 |
| GET | `/api/exclusions` | 排除列表 |
| POST | `/api/exclusions/add` | 添加排除项 |
| POST | `/api/exclusions/remove` | 移除排除项 |
| GET | `/api/chat-sources` | 群/好友列表（NapCat） |
| GET | `/api/claude-progress` | Claude Code 进度 |
| POST | `/api/memories/delete` | 删除记忆 |
| POST | `/api/notes/delete` | 删除笔记 |
| GET | `/workspace_files/*` | 静态文件服务 |

---

## 配置

完整 18 个环境变量见 `README.md` 配置表。

**核心配置：**
```env
NAPCAT_TOKEN=            # 必填，NapCat 鉴权
LLAMA_MODEL_PATH=        # 必填，模型文件路径
LLAMA_SERVER_PATH=       # 必填，llama-server 路径
DATA_DIR=data            # 数据目录
WEB_PORT=5050            # Web 面板端口
```

---

## 编码规范

### Rust
- 使用 `?` 传播错误，避免 `unwrap()`/`expect()`
- 字符串截断用 `.chars().take(n)`，不要按字节切片
- 新增功能必须注册到 ToolRegistry
- 新增数据需加对应的 Store 和 SQLite 表
- 前端改动同步更新 web/mod.rs API 和 index.html

### 提交
- commit message 英文，描述变更
- `changelog/` 下添加对应记录（中文）

### Prompt
- 保持原则式而非规则式
- 不要在 prompt 中写具体例子（会诱导 LLM 编造）
- 说要做的事必须输出 `<tool_call>`，不能只在回复里说"已通知"

---

## 调试

```bash
# 日志级别
RUST_LOG=qq_assistant=debug cargo run --release

# 仅查看 Web 面板
# http://127.0.0.1:5050

# 直接测试 LLM
curl http://127.0.0.1:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"qwen3.5","messages":[{"role":"user","content":"hi"}]}'

# 测试 Embedding
curl http://127.0.0.1:8082/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"model":"qwen3.5","input":"test"}'
```

## 常见问题

### 二进制锁（拒绝访问）
```bash
# 进程占用 .exe 时编译会失败
powershell Stop-Process -Name qq-assistant -Force
cargo build --release
```

### 端口冲突
Web 面板端口 5050 被占用时自动 fallback 到 5051-5053。

### 中文路径
模型路径含中文时 llama-server 可能不认。建 junction 链接：
```cmd
mklink /J D:\llm D:\桌面\编程作品\Sandy ONE\local_model_provider
```

### 上下文超限
模型上下文 8192 tokens，Agent 循环在超过 6144 token 时自动压缩。
如果仍有溢出，降低 `MAX_TOKENS` 阈值或在 prompt 中减少冗余。

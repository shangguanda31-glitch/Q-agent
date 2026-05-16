# QAgent 依赖详情

---

## 一、核心框架

| 依赖 | 版本 | 用途 |
|------|------|------|
| Rust | 2021 edition | 主编程语言 |
| tokio | 1.x | 异步运行时 |
| axum | 0.7 | Web 框架 |
| rusqlite | 0.31 | SQLite 绑定 |
| reqwest | 0.12 | HTTP 客户端 |
| serde / serde_json | 1.x | 序列化 |
| tower-http | 0.5 | CORS / 静态文件 |
| chrono | 0.4 | 日期时间处理 |
| parking_lot | 0.12 | 高性能 Mutex |
| tracing | 0.1 | 日志 |
| uuid | 1.x | 唯一 ID 生成 |
| async-trait | 0.1 | 异步 trait 支持 |
| async-stream | 0.3 | SSE 流 |
| futures-util | 0.3 | 异步工具 |
| base64 | 0.22 | Base64 编解码 |
| dotenvy | 0.15 | .env 文件加载 |
| anyhow | 1.x | 错误处理 |

---

## 二、LLM 推理引擎

### llama.cpp
LLM 推理后端，负责加载和运行量化模型。

| 项目 | 说明 |
|------|------|
| 仓库 | https://github.com/ggml-org/llama.cpp |
| 可执行文件 | `llama-server.exe` |
| 位置 | `D:/llm/llama-bin/llama-server.exe`（通过 `LLAMA_SERVER_PATH` 配置） |
| 编译后端 | CUDA（GPU 推理） |
| 显存占用 | ~5-6 GB（含 KV cache） |

**启动参数：**
```
--ctx-size 8192        上下文窗口 8K tokens
-ngl 40                GPU 层数（笔记本 5070 8GB）
--embeddings           启用 embedding 功能
--pooling mean         embedding 均值池化
```

### 主模型：Qwen3.5-9B

| 项目 | 值 |
|------|-----|
| 模型 | Qwen3.5-9B |
| 量化 | Q4_K_M |
| 文件 | `Qwen3.5-9B.Q4_K_M.gguf` |
| 大小 | 5.3 GB |
| 来源 | https://huggingface.co/Qwen/ |
| API | `/v1/chat/completions` |
| 生成速度 | 42-48 tokens/s |
| 服务端口 | 8081 |

### 嵌入模型：Qwen3.5-0.8B

| 项目 | 值 |
|------|-----|
| 模型 | Qwen3.5-0.8B |
| 量化 | Q6_K |
| 文件 | `Qwen3.5-0.8B-Q6_K.gguf` |
| 大小 | 610 MB |
| 向量维度 | 1024 |
| API | `/v1/embeddings` |
| 运行方式 | CPU（-ngl 0）|
| 服务端口 | 8082 |

---

## 三、外部服务

### NapCatQQ
QQ 协议桥接，负责接收 QQ 消息和调用 QQ API。

| 项目 | 说明 |
|------|------|
| 仓库 | https://github.com/NapNeko/NapCatQQ |
| 协议 | OneBot v11 |
| WebSocket | ws://127.0.0.1:4447（消息推送）|
| HTTP API | http://127.0.0.1:4444（REST 调用）|
| 鉴权 | Bearer Token（通过 `NAPCAT_TOKEN` 配置）|

**API 端点：**
- `GET /get_group_list` — 群列表
- `GET /get_friend_list` — 好友列表
- `GET /get_group_notice` — 群公告
- `GET /get_image` — 图片信息
- `GET /get_group_file_url` — 群文件 URL
- `GET /download_file` — 文件下载

### Tesseract OCR
图片文字识别引擎。

| 项目 | 说明 |
|------|------|
| 版本 | 5.x |
| 位置 | `{project_root}/tesseract/tesseract.exe` |
| 语言数据 | `{project_root}/tesseract/tessdata/` |
| 语言包 | `chi_sim`（简体中文）+ `eng`（英文）|
| 配置 | `TESSERACT_PATH` 环境变量 |
| 数据目录 | `TESSDATA_PREFIX` 环境变量 |

### Claude Code
复杂任务执行引擎。

| 项目 | 说明 |
|------|------|
| CLI 命令 | `claude`（npm 全局安装）|
| 版本 | 2.1.139 |
| 安装方式 | `npm install -g @anthropic-ai/claude-code` |
| 运行模式 | `-p`（非交互式）|
| 输出格式 | `stream-json`（实时流）|
| 超时 | 1800 秒（30 分钟）|
| 并发 | 最多 2 个（Semaphore 队列）|
| 工作目录 | `claude_workspace/` |
| 配置文件 | `claude_workspace/claude.json` |

**Claude Code 配置（claude.json）：**
```json
{
  "enabledPlugins": {
    "frontend-design@claude-plugins-official": true,
    "rust-analyzer-lsp@claude-plugins-official": true
  },
  "env": {
    "ANTHROPIC_BASE_URL": "https://maas-coding-api.cn-huabei-1.xf-yun.com/anthropic",
    "ANTHROPIC_MODEL": "astron-code-latest"
  }
}
```

---

## 四、存储

| 项目 | 说明 |
|------|------|
| 数据库引擎 | SQLite 3 |
| 模式 | WAL（Write-Ahead Logging）|
| 并发控制 | `parking_lot::Mutex` |
| 文件位置 | `data/data.db`（由 `DATA_DIR` 配置）|
| 图片缓存 | `image_cache/`（相对运行目录）|

**数据库表：**

| 表名 | 用途 | 关键字段 |
|------|------|---------|
| `events` | 消息事件日志（上限 500 条）| time, message_type, raw_text, analysis |
| `schedules` | 日程 | title, time, time_parsed, description, status |
| `memories` | 语义记忆（含 embedding）| content, tags, embedding (1024d) |
| `notes` | 笔记 | content, speaker, source, group_id |
| `exclusions` | 排除列表 | exclude_type, target_id |

---

## 五、环境变量完整列表

```env
# NapCat 配置
NAPCAT_WS_URL=ws://127.0.0.1:4447
NAPCAT_HTTP_URL=http://127.0.0.1:4444
NAPCAT_TOKEN=your_token_here

# Web 面板
WEB_PORT=5050

# 数据目录
DATA_DIR=data

# LLM 服务
LLM_URL=http://127.0.0.1:8081
EMBED_URL=http://127.0.0.1:8082
LLM_MODEL=qwen3.5-9b

# 模型文件
LLAMA_SERVER_PATH=llama-server
LLAMA_MODEL_PATH=models/qwen3.5-9b-q4_k_m.gguf
EMBED_MODEL_PATH=models/qwen3.5-0.8b-q6_k.gguf
LLAMA_GPU_LAYERS=40

# Agent 循环
MAX_TOOL_ITERATIONS=10

# Claude Code
CLAUDE_CODE_ENABLED=true
CLAUDE_CODE_TIMEOUT=1800
CLAUDE_WORKING_DIR=claude_workspace

# Tesseract OCR
TESSERACT_PATH=
TESSDATA_PREFIX=tesseract/tessdata
```

---

## 六、可选依赖

| 依赖 | 必须？| 说明 |
|------|-------|------|
| Tesseract 5.x | 否 | 不用则 OCR 工具不可用 |
| Claude Code CLI | 否 | 不用则 claude_code 工具不可用 |
| CUDA Toolkit | 是 | GPU 推理需要（已预装）|
| NapCatQQ | 是 | QQ 消息收发需要 |

# API 参考

---

## 一、Web API（QAgent Web 面板）

基础 URL：`http://127.0.0.1:5050`

> **注意**：当前所有 Web API 无认证机制（Issue #34）。

### 1.1 页面

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/` | Web 面板首页（SPA） |

### 1.2 实时事件

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/events` | SSE 流，实时推送已处理消息事件 |

事件格式：
```json
{
  "time": "2026-05-16 12:00:00",
  "message_type": "group",
  "group_id": 123456,
  "user_id": 789012,
  "sender_name": "用户",
  "raw_text": "消息内容",
  "has_image": false,
  "has_file": false,
  "file_name": null,
  "analysis": null
}
```

### 1.3 消息历史

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/history` | 最近 100 条已处理消息 |

返回 `ProcessedEvent[]`。

### 1.4 日程管理

| 方法 | 路径 | 请求体 | 说明 |
|------|------|--------|------|
| GET | `/api/schedules` | — | 获取所有日程 |
| POST | `/api/schedules/done` | `{"id": "uuid"}` | 标记日程完成 |
| POST | `/api/schedules/delete` | `{"id": "uuid"}` | 删除日程 |

日程字段：
```json
{
  "id": "uuid",
  "title": "开会",
  "time": "明天下午5点",
  "time_parsed": "2026-05-17 17:00",
  "description": "地点：会议室",
  "source": "QQ消息",
  "source_user": "LLM提取",
  "status": "pending",
  "created_at": "2026-05-16T12:00:00+08:00"
}
```

### 1.5 笔记

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/notes` | 获取所有笔记 |
| POST | `/api/notes/delete` | 删除笔记 `{"id": "uuid"}` |

### 1.6 记忆

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/memories` | 获取所有记忆 |
| POST | `/api/memories/delete` | 删除记忆 `{"id": "uuid"}` |

### 1.7 工作区文件

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/workspace` | 列出 `claude_workspace/` 文件 |
| GET | `/api/workspace/{path}` | 获取文件内容（文本文件） |
| GET | `/workspace_files/*` | 静态文件服务 |

### 1.8 排除列表

| 方法 | 路径 | 请求体 | 说明 |
|------|------|--------|------|
| GET | `/api/exclusions` | — | 查看所有排除项 |
| POST | `/api/exclusions/add` | `{"exclude_type": "group", "target_id": 123456}` | 添加排除 |
| POST | `/api/exclusions/remove` | `{"exclude_type": "group", "target_id": 123456}` | 移除排除 |
| GET | `/api/chat-sources` | — | 从 NapCat 拉取群/好友列表（含排除状态） |

### 1.9 Claude Code

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/claude-progress` | Claude Code 当前进度文本 |

---

## 二、NapCat HTTP API（QQ 协议桥接）

基础 URL：`http://127.0.0.1:4444`
鉴权：`Authorization: Bearer {token}`

### 2.1 群相关

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/get_group_list` | 获取群列表 |
| POST | `/get_group_notice` | 获取群公告 `{"group_id": 123456}` |
| POST | `/get_group_file_url` | 获取群文件 URL `{"group_id": 123, "file_id": "...", "bus_id": 0}` |

### 2.2 好友相关

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/get_friend_list` | 获取好友列表 |

### 2.3 图片

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/get_image` | 获取图片信息 `{"file": "..."}` |
| GET | 外部 URL | 下载文件（通过 `download_file` 代理） |

---

## 三、LLM API（llama-server）

基础 URL：`http://127.0.0.1:8081`

### 3.1 聊天补全

```
POST /v1/chat/completions
Content-Type: application/json
```

```json
{
  "model": "qwen3.5-9b",
  "messages": [
    {"role": "system", "content": "你是 QQ 助理..."},
    {"role": "user", "content": "今天晚上开会"}
  ],
  "temperature": 0.3,
  "max_tokens": 4096
}
```

响应：
```json
{
  "choices": [{"message": {"content": "已创建日程..."}}],
  "usage": {"prompt_tokens": 100, "completion_tokens": 50}
}
```

### 3.2 Embedding

```
POST /v1/embeddings
Content-Type: application/json
```

```json
{
  "model": "qwen3.5-9b",
  "input": "要搜索的文本"
}
```

响应：
```json
{
  "data": [{"embedding": [0.1, 0.2, ...]}]
}
```

> embedding 服务默认使用 0.8B 模型（端口 8082），如果模型文件不存在则回退到主 LLM。

---

## 四、Agent 工具 API（内部）

LLM 通过 `<tool_call>` XML 标签调用工具：

```xml
<tool_call>
{"name": "notify_user", "arguments": {"title": "提醒", "body": "内容"}}
</tool_call>
```

### 工具列表

| 工具名 | 参数 | 说明 |
|--------|------|------|
| `notify_user` | `{title, body}` | Windows 桌面通知 |
| `schedule_create` | `{title, time?, info?}` | 创建日程（自动中文时间解析）|
| `schedule_list` | `{}` | 列出所有日程 |
| `schedule_update` | `{id?, title?, time?, info}` | 更新或自动创建日程 |
| `claude_code` | `{prompt}` | 复杂任务 → Claude Code CLI |
| `ocr_image` | `{image_path}` | Tesseract 中英文 OCR |
| `note_take` | `{content, speaker, source}` | 记录笔记 |
| `remember` | `{content}` | 语义记忆写入（生成 embedding）|
| `recall` | `{query}` | 语义记忆检索（余弦相似度）|
| `qq_read` | `{action, group_id?}` | QQ 群公告等信息 |

### ToolResult 格式

工具执行结果通过 `<tool_result>` 回传给 LLM：

```xml
<tool_result name="notify_user">
工具 notify_user 执行成功:
已发送通知
</tool_result>
```

---

## 五、WebSocket 事件（NapCat → QAgent）

连接：`ws://127.0.0.1:4447/?access_token={token}`

### OneBot v11 消息格式

```json
{
  "post_type": "message",
  "message_type": "group",
  "group_id": 123456,
  "user_id": 789012,
  "sender": {
    "nickname": "用户",
    "card": "群名片"
  },
  "self_id": 1704028969,
  "message": [
    {"type": "text", "data": {"text": "你好"}},
    {"type": "at", "data": {"qq": "1704028969"}}
  ]
}
```

### 消息段类型

| type | data 字段 | 说明 |
|------|-----------|------|
| `text` | `text` | 文本 |
| `image` | `file`, `url` | 图片 |
| `at` | `qq` | @某人 |
| `reply` | `id` | 回复消息 |
| `face` | `id` | QQ 表情 |
| `file` | `file`, `name`, `size` | 文件 |

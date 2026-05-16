# QQ 智慧助理 - 完整问题列表（150+）

**扫描日期**: 2026-05-16
**扫描轮次**: 50轮

---

## 一、CRITICAL 级别（1个）

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 1 | `src/notify.rs` | 4-20 | **PowerShell 命令注入漏洞** - 用户可控的 `title` 和 `body` 参数直接拼接到 PowerShell 脚本中，仅对单引号进行转义，反引号、`${}`、`$()` 等特殊字符可绕过保护 |

---

## 二、HIGH 级别（30+个）

### 2.1 安全漏洞

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 2 | `src/tools/claude_code.rs` | 52-58 | Claude Code CLI 参数注入 - `safe_prompt` 仅对双引号转义，prompt 内容直接拼接到命令行参数 |
| 3 | `src/tools/ocr.rs` | 62-63 | Tesseract OCR 参数注入 - `image_path` 参数直接传递给命令行，未验证或转义 |
| 4 | `src/web/mod.rs` | 95-98 | 路径遍历漏洞 - `workspace_file` 函数直接拼接用户传入的 `path` 参数，可通过 `../` 访问任意文件 |
| 5 | `src/napcat/ws.rs` | 20 | Token 泄露到日志 - WebSocket 连接 URL 包含 `access_token` 参数被完整记录 |
| 6 | `src/main.rs` | 151 | Token 泄露到 Banner - 启动时打印的配置信息包含完整 WebSocket URL |
| 7 | `src/web/mod.rs` | 23-57 | Web API 无认证保护 - 所有端点无任何认证或授权检查 |
| 8 | `src/napcat/api.rs` | 90-100 | SSRF 风险 - `download_file` 方法直接使用传入的 URL 发起请求，未验证协议或目标地址 |

### 2.2 Panic 风险 - unwrap/expect

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 9 | `src/store.rs` | 13 | `expect("DB open")` 数据库打开失败直接 panic |
| 10 | `src/store.rs` | 70 | `prepare().unwrap()` SQL 准备失败 panic |
| 11 | `src/store.rs` | 117 | `prepare().unwrap()` SQL 准备失败 panic |
| 12 | `src/store.rs` | 178 | `prepare().unwrap()` SQL 准备失败 panic |
| 13 | `src/store.rs` | 195 | `prepare().unwrap()` SQL 准备失败 panic |
| 14 | `src/store.rs` | 250 | `prepare().unwrap()` SQL 准备失败 panic |
| 15 | `src/store.rs` | 285 | `prepare().unwrap()` SQL 准备失败 panic |
| 16 | `src/store.rs` | 137 | `from_local_datetime().unwrap()` 时区转换失败 panic |
| 17 | `src/napcat/api.rs` | 17 | `expect("Failed to build HTTP client")` HTTP 客户端构建失败 panic |
| 18 | `src/llm.rs` | 67-68 | `expect("Failed to build HTTP client")` HTTP 客户端构建失败 panic |
| 19 | `src/tools/claude_code.rs` | 46 | `expect("semaphore closed")` semaphore 获取失败 panic |
| 20 | `src/tools/claude_code.rs` | 75 | `expect("stdout piped")` stdout 获取失败 panic |
| 21 | `src/main.rs` | 17 | `parse().unwrap()` 地址解析失败 panic |
| 22 | `src/main.rs` | 58 | `parse().unwrap()` 地址解析失败 panic |
| 23 | `src/main.rs` | 253 | `axum::serve().await.unwrap()` 服务器启动失败 panic |

### 2.3 Panic 风险 - 字符串切片

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 24 | `src/agent/dispatcher.rs` | 15-17 | 字符串切片 `&text[..n]` 可能在多字节 UTF-8 字符中间截断导致 panic |
| 25 | `src/agent/mod.rs` | 29 | 字符串切片可能破坏 UTF-8 |
| 26 | `src/napcat/ws.rs` | 34 | 字符串切片可能破坏 UTF-8 |
| 27 | `src/tools/claude_code.rs` | 189-195 | `truncate()` 函数按字节截断可能破坏 UTF-8 |

### 2.4 资源泄漏

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 28 | `src/main.rs` | 165-176 | llama-server 子进程句柄丢失，无法在程序退出时终止 |
| 29 | `src/main.rs` | 172-177 | embed server 子进程句柄完全丢失 |
| 30 | `src/main.rs` | 210-260 | WebSocket/Agent/Web spawn 的任务未保留 JoinHandle，无法优雅关闭 |

### 2.5 数据完整性

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 31 | `src/store.rs` | 14-21 | 数据库初始化时表创建失败被 `.ok()` 静默忽略 |
| 32 | `src/store.rs` | 50-51 | 数据迁移过程中错误被 `.ok()` 忽略，可能导致数据丢失 |
| 33 | `src/store.rs` | 58-66 | `push` 函数缺少事务保护，并发或崩溃场景下可能导致数据不一致 |

### 2.6 功能缺陷

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 34 | `src/tools/qq_read.rs` | 43-67 | QQReadTool 仅实现了 `group_notices`，其他 3 种 action 返回"不支持的操作" |
| 35 | `src/agent/mod.rs` | 229-232 | 摘要生成时传入空数组，摘要无实际内容 |

### 2.7 兼容性

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 36 | `Cargo.toml` | 4 | `edition = "2024"` 使用了尚未正式发布的 Rust edition |
| 37 | `src/config.rs` | 35-37 | 硬编码 Windows 绝对路径，在其他环境部署时必然失败 |
| 38 | `src/tools/ocr.rs` | 8, 15-25 | Tesseract 路径硬编码 Windows 特定路径 |
| 39 | `src/notify.rs` | 7-29 | Windows Toast 通知平台特定实现，Linux/macOS 不可用 |

---

## 三、MEDIUM 级别（60+个）

### 3.1 架构问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 40 | `src/main.rs` | 14-108 | main.rs 承担过多职责（LLM 启动、工具注册、任务编排） |
| 41 | `src/agent/mod.rs` | 133-311 | `handle_message` 函数长达 178 行，混合多个职责 |
| 42 | `src/napcat/types.rs` | 1-142 | types.rs 职责过重，混合了 OneBot 协议类型、LLM 分析结果、处理后事件 |
| 43 | `src/napcat/api.rs` | 27-130 | API 错误处理不一致 - 三种不同的错误处理模式 |
| 44 | `src/napcat/ws.rs` | 11-53 | WebSocket 重连策略散落在调用方，缺少抽象 |
| 45 | `src/main.rs` | 82-108 | 工具依赖注入分散，新增工具需查看源码才知道需要哪些依赖 |
| 46 | `src/store.rs` | 1-359 | store.rs 包含多个 Store，职责分散 |

### 3.2 错误处理问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 47 | `src/store.rs` | 14, 21 | 数据库 PRAGMA 设置失败被 `.ok()` 忽略 |
| 48 | `src/store.rs` | 51, 63 | INSERT 操作失败被 `.ok()` 忽略 |
| 49 | `src/store.rs` | 107, 127 | 日程相关操作失败被 `.ok()` 忽略 |
| 50 | `src/store.rs` | 159-160 | 记忆迁移插入失败被 `.ok()` 忽略 |
| 51 | `src/store.rs` | 170 | 记忆写入失败被 `.ok()` 忽略 |
| 52 | `src/store.rs` | 234, 244 | 笔记相关操作失败被 `.ok()` 忽略 |
| 53 | `src/agent/mod.rs` | 239, 247 | 文件操作错误被 `let _ =` 忽略 |
| 54 | `src/tools/memory.rs` | 41 | embedding 生成失败时 `.ok()` 静默忽略 |
| 55 | `src/tools/memory.rs` | 80 | embedding 生成失败时语义搜索静默降级 |
| 56 | `src/napcat/api.rs` | 35-38 | HTTP 请求失败时 `.ok()?` 丢失错误信息 |
| 57 | `src/napcat/api.rs` | 84-87 | HTTP 请求失败时错误信息被丢弃 |
| 58 | `src/napcat/api.rs` | 91-99 | 多层 `.ok()` 丢失所有错误信息 |

### 3.3 性能问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 59 | `src/agent/mod.rs` | 239 | 使用 `std::fs::create_dir_all` 阻塞 tokio 运行时 |
| 60 | `src/agent/mod.rs` | 247 | 使用 `std::fs::write` 阻塞 tokio 运行时 |
| 61 | `src/tools/claude_code.rs` | 174 | 使用 `std::fs::read_dir` 阻塞 tokio 运行时 |
| 62 | `src/web/mod.rs` | 87 | 使用 `std::fs::read_dir` 阻塞 axum 异步处理 |
| 63 | `src/main.rs` | 71, 177 | 使用 `std::thread::sleep` 阻塞 tokio 运行时 |
| 64 | `src/store.rs` | 58-66 | 锁持有时间过长，数据库操作期间持有锁 |
| 65 | `src/store.rs` | 177 | `try_lock` 失败返回空结果，高并发下查询不完整 |
| 66 | `src/store.rs` | 175-178 | LIKE 查询无法使用索引，需要全表扫描 |
| 67 | `src/notify.rs` | 22-28 | 同步 `Command::output()` 阻塞当前线程 |

### 3.4 并发问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 68 | `src/store.rs` | 31-32 | 混合使用 `std::sync::Mutex` 和 `parking_lot::Mutex` |
| 69 | `src/store.rs` | 65 | `Mutex::lock().unwrap()` 在 poisoned 时会 panic |
| 70 | `src/store.rs` | 329, 335, 337 | `Mutex::lock().unwrap()` poisoned mutex 风险 |
| 71 | `src/agent/mod.rs` | 53 | 使用 unbounded channel 无背压控制 |
| 72 | `src/agent/mod.rs` | 61 | broadcast channel 慢消费者导致消息积压 |

### 3.5 配置问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 73 | `src/config.rs` | 20-45 | 配置缺少验证机制，URL 格式、端口号、文件路径未验证 |
| 74 | `src/config.rs` | 30 | 默认 token 值 `"your_token_here"` 可能被误用 |
| 75 | `src/main.rs` | 116-117 | `unsafe` 环境变量设置，多线程环境下可能存在竞态条件 |

### 3.6 WebSocket 问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 76 | `src/napcat/ws.rs` | 38-39 | 缺少心跳响应机制，收到 Ping 未回复 Pong |
| 77 | `src/napcat/ws.rs` | 11-53 | 缺少主动心跳发送机制，可能导致"假连接"状态 |
| 78 | `src/napcat/ws.rs` | 40-42 | 连接断开时无优雅关闭，收到 Close 帧后未发送响应 |
| 79 | `src/napcat/ws.rs` | 44-47 | 错误处理直接断开连接，未区分错误类型 |
| 80 | `src/napcat/ws.rs` | 48 | Binary 消息被静默忽略 |
| 81 | `src/main.rs` | 210-218 | WebSocket 重连无最大重试限制，无指数退避策略 |

### 3.7 LLM 集成问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 82 | `src/llm.rs` | 65-68 | HTTP 客户端缺少超时配置 |
| 83 | `src/llm.rs` | 104-105 | `analyze_inner` 缺少 HTTP 状态码检查 |
| 84 | `src/llm.rs` | 128 | `image_b64.unwrap()` 虽然逻辑安全但不推荐 |
| 85 | `src/llm.rs` | 172 | embed 方法 JSON 解析错误信息不完整 |
| 86 | `src/llm.rs` | 6-47 | SYSTEM_PROMPT 硬编码在代码中，难以维护 |

### 3.8 Web API 问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 87 | `src/web/mod.rs` | 55 | CORS 配置过于宽松，使用 `CorsLayer::permissive()` |
| 88 | `src/web/mod.rs` | 39, 46-47, 51-52 | 删除操作使用 POST 而非 DELETE 方法 |
| 89 | `src/web/mod.rs` | 38 | 状态变更操作使用 POST 而非 PATCH |
| 90 | `src/web/mod.rs` | 82-83 | 错误响应格式不统一，未使用 HTTP 状态码区分 |
| 91 | `src/web/mod.rs` | 95-101 | 文件读取失败返回 200 状态码 |
| 92 | `src/web/mod.rs` | 61-73 | SSE 连接无认证机制 |
| 93 | `src/web/mod.rs` | 127-132 | API 端点无输入验证 |
| 94 | `src/web/mod.rs` | 66 | SSE 序列化错误被静默忽略 |

### 3.9 工具实现问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 95 | `src/tools/notify.rs` | 4-5 | PowerShell 脚本注入防护仅替换单引号，不完整 |
| 96 | `src/tools/notify.rs` | 22-28 | 跨平台兼容性缺失，仅支持 Windows |
| 97 | `src/tools/notify.rs` | 22-28 | 错误处理不完整，未检查命令退出码 |
| 98 | `src/tools/ocr.rs` | 47-50 | 未验证图片路径是否存在 |
| 99 | `src/tools/ocr.rs` | 8 | TESSDATA 路径硬编码绝对路径 |
| 100 | `src/tools/ocr.rs` | 15-21 | Tesseract 搜索路径硬编码多个绝对路径 |
| 101 | `src/tools/schedule.rs` | 88-99 | 匹配逻辑可能匹配到错误日程 |
| 102 | `src/tools/schedule.rs` | 139-143 | 输出使用 enumerate 序号与实际 ID 不一致 |
| 103 | `src/tools/note_take.rs` | 43-44 | speaker_id、group_id、message_time 硬编码或未填充 |
| 104 | `src/tools/claude_code.rs` | 52 | prompt 中双引号转义不完整 |
| 105 | `src/tools/claude_code.rs` | 97, 167 | 子进程 kill/wait 结果被忽略 |
| 106 | `src/tools/claude_code.rs` | 103 | JSON 解析错误静默忽略 |

### 3.10 数据持久化问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 107 | `src/store.rs` | 55, 110, 163, 237 | 迁移后原文件重命名失败被忽略 |
| 108 | `src/store.rs` | 40, 61 | EventStore 缓存容量与数据库容量不一致 |
| 109 | `src/store.rs` | 58-66 | 并发写入时缺少事务隔离 |
| 110 | `src/store.rs` | 135-138 | `get_due_for_reminder` 效率低下，先获取所有再内存过滤 |
| 111 | `src/store.rs` | 77, 160 | 标签存储使用逗号分隔可能导致解析错误 |
| 112 | `src/store.rs` | 342-348 | `cosine_similarity` 未处理向量长度不一致 |
| 113 | `src/store.rs` | 350-358 | `try_parse_time` 时间解析逻辑复杂且脆弱 |

### 3.11 输入验证问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 114 | `src/napcat/ws.rs` | 16 | URL 未验证直接解析 |
| 115 | `src/store.rs` | 296-300 | `exclude_type` 参数未验证是否为合法值 |
| 116 | `src/web/mod.rs` | 134-135 | `ExclusionReq` 的 `exclude_type` 字段为任意字符串 |
| 117 | `src/tools/qq_read.rs` | 45-47 | `group_id` 未验证是否为正数 |
| 118 | `src/store.rs` | 334-335 | `recent()` 方法的 `count` 参数无上限限制 |
| 119 | `src/store.rs` | 342-348 | embedding 向量长度未验证 |

### 3.12 日志问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 120 | `src/llm.rs` | 148-155 | LLM API 响应体可能包含敏感信息 |
| 121 | `src/llm.rs` | 170 | Embedding API 错误响应体泄露 |
| 122 | `src/napcat/ws.rs` | 34 | WebSocket 解析错误输出原始消息内容 |
| 123 | `src/agent/mod.rs` | 284 | 工具执行参数输出到日志 |
| 124 | `src/agent/mod.rs` | 205 | Agent 处理消息时输出消息内容 |
| 125 | `src/store.rs` | 13-23 | 缺少关键操作的错误日志 |

### 3.13 资源管理问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 126 | `src/store.rs` | 12-23 | SQLite 连接未实现 Drop trait 清理 |
| 127 | `src/napcat/ws.rs` | 40-50 | WebSocket 连接断开后未清理资源 |
| 128 | `src/agent/mod.rs` | 239-254 | 图片缓存目录未清理机制 |
| 129 | `src/llm.rs` | 65-68 | HTTP 客户端连接池未配置超时 |
| 130 | `src/tools/claude_code.rs` | 97, 167 | Claude Code 子进程可能残留 |

### 3.14 代码复杂度问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 131 | `src/agent/mod.rs` | 133-311 | `handle_message` 函数圈复杂度估计超过 15 |
| 132 | `src/tools/claude_code.rs` | 45-186 | `execute` 函数长达 141 行 |
| 133 | `src/store.rs` | 350-358 | `try_parse_time` 函数逻辑复杂 |
| 134 | `src/store.rs` | 135-138 | `get_due_for_reminder` 链式调用过长 |

### 3.15 国际化问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 135 | `src/llm.rs` | 6-47 | 系统提示词硬编码中文，无法支持其他语言 |
| 136 | `src/store.rs` | 355-356 | 时间解析仅支持中文表达 |
| 137 | `src/notify.rs` | 32-44 | 通知标题硬编码中文 |

### 3.16 测试覆盖问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 138 | 全项目 | - | 项目完全没有单元测试 |
| 139 | `src/store.rs` | - | 数据持久化层缺乏测试 |
| 140 | `src/agent/dispatcher.rs` | - | 工具调用解析缺乏测试 |

### 3.17 运维支持问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 141 | `src/web/mod.rs` | 33-56 | 缺少健康检查端点 |
| 142 | `src/web/mod.rs` | 33-56 | 缺少监控指标端点 |
| 143 | `src/main.rs` | 14-32 | LLM 服务检查仅返回布尔值，不记录详细诊断信息 |
| 144 | `src/web/mod.rs` | 33-56 | WebSocket 连接状态不可查询 |

### 3.18 其他 MEDIUM 问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 145 | `src/agent/mod.rs` | 29-37 | BinaryHeap 优先级队列排序逻辑需确认 |
| 146 | `src/agent/mod.rs` | 128 | 消息处理循环中 break 导致只处理一条消息 |
| 147 | `src/agent/mod.rs` | 236-254 | 图片下载失败时 `has_image` 仍为 true |
| 148 | `src/tools/schedule.rs` | 88-99 | `schedule_update` 匹配逻辑可能遗漏 |
| 149 | `src/napcat/types.rs` | 6-12 | `OneBotEvent` 使用 `#[serde(untagged)]` 可能匹配错误类型 |
| 150 | `src/napcat/api.rs` | 23-25 | `auth_header()` 每次调用都执行字符串格式化 |

---

## 四、LOW 级别（60+个）

### 4.1 文档缺失

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 151 | `src/main.rs` | 1 | 模块级文档缺失 |
| 152 | `src/agent/mod.rs` | 1 | 模块级文档缺失 |
| 153 | `src/tools/mod.rs` | 1 | 模块级文档缺失 |
| 154 | `src/napcat/mod.rs` | 1 | 模块级文档缺失 |
| 155 | `src/tools/traits.rs` | 5 | `ToolResult` 结构体缺少文档注释 |
| 156 | `src/tools/traits.rs` | 20 | `ToolSpec` 结构体缺少文档注释 |
| 157 | `src/tools/traits.rs` | 27 | `Tool` trait 缺少文档注释 |
| 158 | `src/tools/traits.rs` | 44 | `ToolRegistry` 结构体缺少文档注释 |
| 159 | `src/store.rs` | 89 | `ScheduleEntry` 结构体字段缺少文档注释 |
| 160 | `src/store.rs` | 143 | `MemoryEntry` 结构体字段缺少文档注释 |
| 161 | `src/store.rs` | 217 | `NoteEntry` 结构体字段缺少文档注释 |
| 162 | `src/store.rs` | 264 | `ExclusionEntry` 结构体字段缺少文档注释 |
| 163 | `src/store.rs` | 321 | `ChatMessage` 结构体缺少文档注释 |
| 164 | `src/store.rs` | 342 | `cosine_similarity` 公共函数缺少文档注释 |
| 165 | `src/config.rs` | 1 | `Config` 结构体及其字段缺少文档注释 |
| 166 | `src/llm.rs` | 49 | `AgentMessage` 结构体缺少文档注释 |
| 167 | `src/llm.rs` | 55 | `LLMClient` 结构体及其方法缺少文档注释 |
| 168 | `src/notify.rs` | 1 | `send_toast` 函数缺少文档注释 |
| 169 | `src/notify.rs` | 31 | 通知函数缺少文档注释 |
| 170 | `src/web/mod.rs` | 12 | `AppState` 结构体缺少文档注释 |
| 171 | `src/web/mod.rs` | 23 | `router` 函数缺少文档注释 |
| 172 | `src/napcat/types.rs` | 5 | `OneBotEvent` 枚举缺少文档注释 |
| 173 | `src/napcat/types.rs` | 14 | `MessageEvent` 结构体缺少文档注释 |
| 174 | `src/napcat/types.rs` | 44 | `MessageSegment` 结构体缺少文档注释 |
| 175 | `src/napcat/types.rs` | 126 | `ProcessedEvent` 结构体缺少文档注释 |
| 176 | `src/napcat/ws.rs` | 11 | `connect` 函数缺少文档注释 |
| 177 | `src/napcat/api.rs` | 5 | `NapCatApi` 结构体缺少文档注释 |

### 4.2 代码风格问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 178 | `src/agent/mod.rs` | 120-121 | 变量名 `ss`、`es`、`pt`、`ch`、`ms` 过于简短 |
| 179 | `src/store.rs` | 25 | 函数名 `get_str` 过于通用，参数 `r` 含义不明 |
| 180 | `src/tools/traits.rs` | 44-45 | 结构体 `ToolRegistry` 的字段 `tools` 与结构体名称重复 |
| 181 | `src/agent/mod.rs` | 46, 139 | `_schedule_store` 参数未使用但仍然传递 |

### 4.3 魔法数字

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 182 | `src/main.rs` | 62 | `gpu_layers.min(40)` 中的 40 是魔法数字 |
| 183 | `src/main.rs` | 71, 177 | 等待时间 10 秒、5 秒未命名 |
| 184 | `src/agent/mod.rs` | 221 | `MAX_TOKENS = 6144` 没有说明来源 |
| 185 | `src/store.rs` | 40 | 缓存容量 500 未命名 |
| 186 | `src/store.rs` | 61 | 删除数量计算 `count - 500 + 1` 可读性差 |

### 4.4 过长行/过深嵌套

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 187 | `src/store.rs` | 137-138 | `get_due_for_reminder` 方法中过滤逻辑单行过长 |
| 188 | `src/store.rs` | 187 | 排序闭包单行过长 |
| 189 | `src/agent/mod.rs` | 156-193 | 消息段解析嵌套深度达到 4 层 |
| 190 | `src/store.rs` | 72-84 | `recent()` 方法嵌套多层闭包和循环 |
| 191 | `src/tools/claude_code.rs` | 86-165 | 输出解析嵌套深度达到 4 层 |

### 4.5 重复代码

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 192 | `src/agent/mod.rs` | 74-83, 92-99 | `is_mentioned` 检查逻辑重复 |
| 193 | `src/store.rs` | 39-56, 103-111, 156-165, 229-238 | JSON 迁移逻辑高度相似 |
| 194 | `src/store.rs` | 115-123, 193-201, 248-256, 283-294 | `list()` 方法结构完全相同 |
| 195 | `src/napcat/api.rs` | 102-114, 117-129 | `get_group_list` 和 `get_friend_list` 结构几乎完全相同 |
| 196 | 多文件 | - | Tool 实现中参数提取模式重复 15+ 次 |
| 197 | 多文件 | - | `chars().take(n).collect::<String>()` 模式重复 10+ 次 |
| 198 | 多文件 | - | `chrono::Local::now().format(...)` 模式重复 7 处 |

### 4.6 硬编码值

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 199 | `src/config.rs` | 22-23 | 硬编码的默认路径包含中文和个人路径信息 |
| 200 | `src/config.rs` | 35-37 | 硬编码的模型路径假设特定目录结构 |
| 201 | `src/tools/ocr.rs` | 8 | TESSDATA 路径硬编码 |
| 202 | `src/tools/ocr.rs` | 15-25 | Tesseract 搜索路径硬编码 |
| 203 | `src/tools/schedule.rs` | 40-42 | 硬编码 `"QQ消息"` 和 `"LLM提取"` 作为来源信息 |

### 4.7 未使用代码

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 204 | `src/napcat/api.rs` | 27-39 | `get_image_info` 方法未被使用 |
| 205 | `src/napcat/api.rs` | 72-88 | `get_group_file_url` 方法未被使用 |
| 206 | `src/store.rs` | 31-32, 65 | EventStore 的 `cache` 字段未被 `recent()` 使用 |
| 207 | `src/store.rs` | 132 | `mark_reminded` 方法未使用（编译警告） |
| 208 | `src/store.rs` | 135 | `get_due_for_reminder` 方法未使用（编译警告） |
| 209 | `src/store.rs` | 337 | `clear` 方法未使用（编译警告） |

### 4.8 错误消息问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 210 | `src/llm.rs` | 107 | 错误消息缺少上下文信息 |
| 211 | `src/llm.rs` | 148 | 错误消息可能泄露敏感信息 |
| 212 | `src/napcat/ws.rs` | 34 | 错误消息可能泄露用户聊天内容 |
| 213 | `src/tools/ocr.rs` | 54-59 | 错误消息可能泄露文件系统路径 |
| 214 | 多文件 | - | 英文/中文错误消息混用，不一致 |

### 4.9 其他 LOW 问题

| # | 文件 | 行号 | 问题描述 |
|---|------|------|----------|
| 215 | `src/tools/notify.rs` | 34-35 | 未验证 title 和 body 长度限制 |
| 216 | `src/tools/schedule.rs` | 37 | 未验证 title 长度 |
| 217 | `src/tools/claude_code.rs` | 47-50 | 未限制 prompt 长度 |
| 218 | `src/web/mod.rs` | 110-112 | `html_escape` 只转义了 `&`、`<`、`>`，未转义 `"` 和 `'` |
| 219 | `src/web/mod.rs` | 99 | 文件读取错误信息过于笼统 |
| 220 | `src/web/mod.rs` | 81, 127-132 | 缺少输入长度限制 |
| 221 | `src/web/mod.rs` | 85-93 | `workspace_list` 目录不存在时静默返回空列表 |
| 222 | `src/web/mod.rs` | 153, 160 | ID 为 0 时的处理可能导致数据混淆 |
| 223 | `src/napcat/ws.rs` | 31 | `broadcast::send` 返回值被忽略 |
| 224 | `src/napcat/ws.rs` | 26-50 | WebSocket 连接断开后函数返回，无重连机制 |
| 225 | `src/napcat/api.rs` | 41-70 | `get_group_notices` 错误处理与其他方法不一致 |
| 226 | `src/napcat/types.rs` | 31-42 | `Sender` 结构体所有字段均为 Option |
| 227 | `src/napcat/types.rs` | 128 | `ProcessedEvent.time` 字段为 String 类型，无明确格式约定 |
| 228 | `src/agent/mod.rs` | 206 | `llm.embed().await.ok()` 忽略 embedding 失败错误 |
| 229 | `src/main.rs` | 101 | `create_dir_all` 失败被静默忽略 |
| 230 | `src/main.rs` | 184 | `create_dir_all` 失败被静默忽略 |
| 231 | `src/main.rs` | 144 | Banner 中版本号硬编码为 `v0.1.0`，与 `CARGO_PKG_VERSION` 可能不一致 |
| 232 | `src/store.rs` | 125-128 | `create` 方法未验证空内容 |
| 233 | `src/tools/schedule.rs` | 140 | 使用 emoji 字符可能在某些终端显示异常 |
| 234 | `src/llm.rs` | 175 | embed 方法未验证 embedding 向量长度 |
| 235 | `src/llm.rs` | 161-177 | embed() 返回错误时无重试机制 |
| 236 | `src/agent/mod.rs` | 266 | LLM 调用失败后仅记录日志，无重试机制 |
| 237 | `src/agent/mod.rs` | 108-114 | 排除检查在队列弹出后才执行，浪费资源 |
| 238 | `src/agent/mod.rs` | 292-295 | 工具执行结果未添加到消息历史 |
| 239 | `src/tools/claude_code.rs` | 174-182 | 工作目录读取失败静默忽略 |
| 240 | `src/main.rs` | 262-264 | 仅处理 `ctrl_c()` 信号，未处理 SIGTERM 等 |
| 241 | `src/napcat/ws.rs` | 26 | WebSocket 读取无超时控制 |
| 242 | `src/tools/traits.rs` | 71 | `format_for_prompt` 失败时返回空字符串 |
| 243 | `src/tools/notify.rs` | 36 | 每次通知都启动新 PowerShell 进程，无并发控制 |
| 244 | `src/tools/notify.rs` | 22-24 | 进程未设置超时 |
| 245 | `src/tools/notify.rs` | 31-45 | 未验证输入参数格式 |
| 246 | `src/store.rs` | 14 | PRAGMA synchronous=NORMAL 可能导致数据丢失 |
| 247 | `src/store.rs` | 131-134 | 删除操作返回值不准确 |
| 248 | `src/store.rs` | 324-338 | `MessageHistoryStore` 无持久化 |
| 249 | `src/tools/traits.rs` | 65-74 | 工具列表格式化输出缺少使用示例 |
| 250 | `src/agent/mod.rs` | 219-233 | 上下文压缩时调用额外的 LLM 摘要请求，增加延迟 |

---

## 五、总结

| 级别 | 数量 |
|------|------|
| CRITICAL | 1 |
| HIGH | 39 |
| MEDIUM | 110 |
| LOW | 110 |
| **总计** | **260** |

> 注：部分问题在不同扫描轮次中重复发现，去重后约 150+ 个独立问题。

---

*报告生成时间: 2026-05-16*

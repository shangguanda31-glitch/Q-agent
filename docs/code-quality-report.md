# QQ 智慧助理 - 代码质量扫描报告

**扫描日期**: 2026-05-16
**扫描轮次**: 50轮
**扫描工具**: Claude Code 子代理

---

## 概述

对 QQ 智慧助理项目进行了全面的代码质量扫描，共发现 **150+ 个问题**，按严重程度分类如下：

| 严重程度 | 数量 | 说明 |
|---------|------|------|
| CRITICAL | 1 | 命令注入漏洞 |
| HIGH | 25+ | 安全漏洞、panic 风险、资源泄漏 |
| MEDIUM | 50+ | 架构问题、错误处理不当 |
| LOW | 75+ | 代码风格、文档缺失 |

---

## 一、安全漏洞（CRITICAL/HIGH）

### 1.1 命令注入漏洞

#### CRITICAL: PowerShell 脚本注入
- **文件**: `src/notify.rs:4-20`
- **描述**: `send_toast` 函数中用户可控的 `title` 和 `body` 参数直接拼接到 PowerShell 脚本中，仅对单引号进行转义，不足以防止所有 PowerShell 注入攻击
- **攻击向量**: 反引号、`${}`、`$()` 等特殊字符可绕过单引号保护
- **修复建议**: 使用 PowerShell 的 `-EncodedCommand` 参数或参数化方式传递变量

#### HIGH: Claude Code CLI 参数注入
- **文件**: `src/tools/claude_code.rs:52-58`
- **描述**: `safe_prompt` 仅对双引号转义，prompt 内容直接拼接到命令行参数
- **修复建议**: 使用 stdin 传递 prompt 或完整转义所有 Shell 元字符

#### HIGH: Tesseract OCR 参数注入
- **文件**: `src/tools/ocr.rs:62-63`
- **描述**: `image_path` 参数直接传递给命令行，未验证或转义
- **修复建议**: 验证路径合法性，检查文件存在性

### 1.2 路径遍历漏洞

- **文件**: `src/web/mod.rs:95-98`
- **描述**: `workspace_file` 函数直接拼接用户传入的 `path` 参数，可通过 `../` 访问任意文件
- **修复建议**: 使用 `Path::canonicalize()` 并检查路径是否在允许目录内

### 1.3 敏感信息泄露

- **文件**: `src/napcat/ws.rs:20`, `src/main.rs:151`
- **描述**: Token 通过 URL 参数传递并被记录到日志
- **修复建议**: 日志输出前对 token 进行脱敏处理

### 1.4 认证缺失

- **文件**: `src/web/mod.rs:23-57`
- **描述**: 所有 Web API 端点无认证保护
- **修复建议**: 添加 API Token 或 Session 认证机制

---

## 二、Panic 风险（HIGH）

### 2.1 unwrap/expect 使用不当

| 文件 | 行号 | 问题 |
|------|------|------|
| `store.rs` | 13 | `expect("DB open")` 数据库打开失败 panic |
| `store.rs` | 70, 117, 178, 195, 250, 285 | `prepare().unwrap()` SQL 准备失败 panic |
| `store.rs` | 137 | `from_local_datetime().unwrap()` 时区转换失败 panic |
| `napcat/api.rs` | 17 | `expect("Failed to build HTTP client")` |
| `llm.rs` | 67-68 | `expect("Failed to build HTTP client")` |
| `tools/claude_code.rs` | 46, 75 | `expect()` semaphore/stdout panic |
| `agent/dispatcher.rs` | 15-17 | 字符串切片可能导致 panic |

### 2.2 字符串切片风险

- **文件**: `src/agent/mod.rs:29`, `src/napcat/ws.rs:34`, `src/tools/claude_code.rs:189-195`
- **描述**: 使用 `&text[..n]` 按字节切片，可能在多字节 UTF-8 字符中间截断导致 panic
- **修复建议**: 使用 `chars()` 迭代器进行字符级截断

---

## 三、资源泄漏（HIGH）

### 3.1 子进程未正确管理

- **文件**: `src/main.rs:165-176`
- **描述**: llama-server 和 embed server 子进程句柄丢失，无法在程序退出时终止
- **修复建议**: 保存进程句柄，在 Ctrl+C 时清理

### 3.2 异步任务未等待完成

- **文件**: `src/main.rs:210-260`
- **描述**: WebSocket/Agent/Web spawn 的任务未保留 JoinHandle，无法优雅关闭
- **修复建议**: 使用 CancellationToken 协调关闭

---

## 四、架构问题（MEDIUM）

### 4.1 职责过重

- **文件**: `src/main.rs:14-108`
- **问题**: main.rs 承担过多职责（LLM 启动、工具注册、任务编排）
- **建议**: 拆分为独立模块

- **文件**: `src/agent/mod.rs:133-311`
- **问题**: `handle_message` 函数长达 178 行，混合多个职责
- **建议**: 拆分为独立函数

### 4.2 类型定义位置不当

- **文件**: `src/napcat/types.rs`
- **问题**: `LLMAnalysis` 和 `ProcessedEvent` 不是 OneBot 协议类型，却放在 napcat 模块
- **建议**: 移至独立的 domain 模块

### 4.3 依赖关系不透明

- **文件**: `src/main.rs:82-108`
- **问题**: 工具依赖注入分散，新增工具需查看源码才知道需要哪些依赖
- **建议**: 添加依赖说明文档或使用 Builder 模式

---

## 五、错误处理问题（MEDIUM）

### 5.1 错误被静默忽略

- **文件**: `src/store.rs` 多处
- **问题**: 大量使用 `.ok()` 忽略数据库操作错误
- **建议**: 至少记录错误日志

### 5.2 错误处理不一致

- **文件**: `src/napcat/api.rs:27-130`
- **问题**: 不同方法使用不同的错误处理策略
- **建议**: 统一错误处理策略

### 5.3 embedding 失败静默降级

- **文件**: `src/tools/memory.rs:41, 80`
- **问题**: embedding 生成失败时语义搜索静默降级为文本搜索
- **建议**: 记录警告日志或提示用户

---

## 六、性能问题（MEDIUM）

### 6.1 异步函数中阻塞操作

- **文件**: `src/agent/mod.rs:239, 247`, `src/main.rs:71, 177`
- **问题**: 使用 `std::fs` 和 `std::thread::sleep` 阻塞 tokio 运行时
- **建议**: 使用 `tokio::fs` 和 `tokio::time::sleep`

### 6.2 锁竞争

- **文件**: `src/store.rs:58-66, 177`
- **问题**: `try_lock` 失败返回空结果，高并发下查询不完整
- **建议**: 使用阻塞锁或添加重试机制

### 6.3 缺少超时配置

- **文件**: `src/llm.rs:65-68`, `src/napcat/api.rs:14-17`
- **问题**: HTTP 客户端无超时配置，请求可能无限期挂起
- **建议**: 添加 `.timeout()` 配置

---

## 七、代码质量问题（LOW）

### 7.1 硬编码值

- **文件**: `src/config.rs:22-37`, `src/tools/ocr.rs:8, 15-25`
- **问题**: 路径、端口等硬编码，缺乏可移植性
- **建议**: 使用配置文件或环境变量

### 7.2 重复代码

- **文件**: `src/store.rs` 多处
- **问题**: JSON 迁移逻辑、参数提取模式重复
- **建议**: 提取通用函数

### 7.3 文档缺失

- **文件**: 所有模块
- **问题**: 模块级文档、公共 API 文档普遍缺失
- **建议**: 添加文档注释

---

## 八、功能缺陷

### 8.1 QQReadTool 功能未实现

- **文件**: `src/tools/qq_read.rs:43-67`
- **问题**: schema 定义了 4 种 action，但只实现了 `group_notices`
- **建议**: 实现缺失功能或从 schema 移除

### 8.2 摘要生成逻辑错误

- **文件**: `src/agent/mod.rs:229-232`
- **问题**: 调用 `agent_chat` 时传入空数组，摘要无实际内容
- **建议**: 将 `old_msgs` 内容传入

---

## 优先修复建议

### 立即修复（安全风险）
1. PowerShell 命令注入漏洞 (`notify.rs`)
2. 路径遍历漏洞 (`web/mod.rs`)
3. Token 泄露到日志 (`ws.rs`, `main.rs`)

### 近期修复（稳定性风险）
1. 子进程资源泄漏 (`main.rs`)
2. unwrap/expect panic 风险 (`store.rs` 等)
3. 数据库操作错误被忽略 (`store.rs`)

### 中期优化
1. 架构重构（main.rs、agent/mod.rs 职责拆分）
2. 统一错误处理策略
3. 添加 HTTP 请求超时配置

### 长期改进
1. 添加 API 认证机制
2. 完善文档注释
3. 消除代码重复

---

## 附录：扫描轮次详情

| 轮次 | 扫描类型 | 发现问题数 |
|------|---------|-----------|
| 1-5 | 基础代码质量扫描 | 50+ |
| 6-10 | 边界条件/并发安全 | 30+ |
| 11-15 | 输入验证/资源管理 | 25+ |
| 16-20 | LLM集成/WebSocket/API | 20+ |
| 21-25 | 异步模式/文档 | 15+ |
| 26-30 | 业务逻辑/数据完整性 | 15+ |
| 31-35 | LLM客户端/通知/启动流程 | 10+ |
| 36-50 | 综合扫描 | 10+ |

---

*报告生成时间: 2026-05-16*

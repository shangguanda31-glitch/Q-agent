# QAgent — 项目规范

**作者**: Orin Voss
**仓库**: https://github.com/OrinVoss/Q-agent
**技术栈**: Rust + Qwen3.5-9B + SQLite + Axum + NapCatQQ

## 架构概览

```
NapCatQQ WebSocket → Agent Loop (LLM 判断 + 工具调度) → Web 面板
                                ↓
                     ExclusionStore | SQLite | Memory | Notify
                                ↓
                     Claude Code (子进程, stream-json)
```

## 关键路径

| 路径 | 说明 |
|------|------|
| `src/main.rs` | 入口：启动 LLM → WebSocket → Agent → Web |
| `src/agent/mod.rs` | AgentLoop：消息处理 + LLM 循环 + 工具调度 |
| `src/agent/prompt.rs` | System prompt 构建（原则式，非规则式） |
| `src/agent/dispatcher.rs` | 解析 `<tool_call>` XML 标签 |
| `src/tools/traits.rs` | Tool trait + ToolRegistry |
| `src/tools/*.rs` | 10 个工具实现 |
| `src/napcat/` | NapCatQQ WebSocket + HTTP API |
| `src/llm.rs` | LLM + Embedding 客户端 |
| `src/store.rs` | SQLite 存储（Event/Schedule/Memory/Note/Exclusion） |
| `src/web/mod.rs` | Web 面板 + API 端点 |
| `src/web/static/index.html` | 前端单页应用 |
| `changelog/` | 修改记录（每次提交必填） |
| `docs/` | 架构/工具/完善计划/远景文档 |

## 开发准则

### 修改记录
每次代码变更必须在 `changelog/` 下创建 Markdown 文件：

```markdown
# 标题
**日期**: YYYY-MM-DD
**类型**: 功能新增 | Bug 修复 | 重构 | 文档 | 配置
## 变更内容
- 做了什么，改了哪些文件
## 原因
- 为什么改
```

### 代码规范
- Rust edition 2021（不要用 2024，尚未稳定）
- 4 空格缩进，遵循现有风格
- 新增功能必须注册到 ToolRegistry
- 新增数据需加对应的 Store 和 SQLite 表
- 前端改动同步更新 web/mod.rs API 和 index.html
- **避免 `unwrap()` 和 `expect()`**，使用 `?` 传播错误
- 字符串截断用 `.chars().take(n)`，不要按字节切片

### 数据存储
- 统一使用 SQLite（data.db, WAL 模式）
- JSON 文件已废弃，首次启动自动迁移
- 模型文件保存在 D:/llm/models/（通过 junction 链接）
- ExclusionStore 管理被排除的群/用户

## 已知教训

### 安全
- ⚠️ **严禁** 将 Token/密码/密钥写入文档、README、changelog 或任何公开文件
- 配置默认值用占位符（如 `your_token_here`），不用真实值
- 敏感信息仅通过环境变量传入，不硬编码
- `claude_workspace/` 已 gitignore，API token 不会泄露
- Web 面板 API 当前无认证（已知问题 #34）

### LLM 行为
- prompt 保持原则式而非规则式，相信 LLM 判断力
- 不要在 prompt 中写具体地名/人名例子（会诱导 LLM 编造）
- 说要做的事必须真的输出 `<tool_call>`，不能只在回复里说"已通知"
- 更新日程用 schedule_update + id 精确匹配，不要靠标题
- schedule_list 返回的 ID 可供后续 schedule_update 使用

### Claude Code
- 使用 `--output-format stream-json --include-partial-messages --verbose`
- 不要用 `--max-iter`（已废弃），用 `--effort max`
- 默认超时 1800s，最多 2 个并发
- 工作目录 `claude_workspace/`，配置在 `claude.json`

## 调试

| 项目 | 地址 |
|------|------|
| 日志级别 | `RUST_LOG=qq_assistant=debug` |
| Web 面板 | `http://127.0.0.1:5050` |
| LLM API | `http://127.0.0.1:8081` |
| Embed API | `http://127.0.0.1:8082` |
| NapCat HTTP | `http://127.0.0.1:4444` |
| GitHub Issues | `https://github.com/OrinVoss/Q-agent/issues` |

## 当前 Issue 优先级

共 50 个 open issues (#10-#59)：

- **CRITICAL**: #10 PowerShell 命令注入
- **HIGH**: #11-#15 安全/Panic/泄漏/数据/兼容
- **MEDIUM**: #16-#39 稳定性/工具/Web/LLM/配置
- **LOW**: #40-#59 重构/测试/硬编码/重复代码

完整清单：`docs/issues/issue-list-full.md`
完善计划：`docs/project/polish-plan.md`
远景规划：`docs/project/vision.md`

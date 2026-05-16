# QAgent — 项目规范

**作者**: Orin Voss
**仓库**: https://github.com/OrinVoss/Q-agent
**技术栈**: Rust + Qwen3.5-9B + SQLite + Axum + NapCatQQ

## 架构概览

```
NapCatQQ WebSocket -> 优先级队列 -> 排除过滤 -> Agent Loop (LLM + 工具) -> Web
                                         |
                              ExclusionStore | SQLite (6 表)
                                         |
                              Claude Code (stream-json) | Tesseract OCR
```

## 关键路径

| 路径 | 说明 |
|------|------|
| `src/main.rs` | 入口：启动 LLM -> WebSocket -> Agent -> Web |
| `src/config.rs` | 环境变量配置（18 个） |
| `src/agent/` | AgentLoop + prompt + tool_call 解析 |
| `src/tools/` | 10 个工具（traits.rs + 各工具实现） |
| `src/napcat/` | NapCatQQ WebSocket + HTTP API |
| `src/llm.rs` | LLM + Embedding 客户端 |
| `src/store.rs` | SQLite 存储（6 表）|
| `src/web/` | Web 面板 + 18 个 API 端点 |
| `changelog/` | 修改记录（每次提交必填） |
| `docs/` | 技术/项目/问题分类文档 |

## 开发准则

### 修改记录
每次代码变更必须在 `changelog/` 下创建 Markdown 文件，标明日期、类型、变更内容和原因。

### 代码规范
- Rust edition 2021（不要用 2024）
- 避免 `unwrap()`/`expect()`，用 `?` 传播错误
- 字符串截断用 `.chars().take(n)`，不要按字节切片
- 新增功能必须注册到 ToolRegistry
- 新增数据需加对应的 Store 和 SQLite 表
- 前端改动同步更新 web/mod.rs API 和 index.html

### 数据存储
- SQLite WAL 模式（data.db），6 张表
- JSON 文件已废弃，首次启动自动迁移
- `claude_workspace/` 已 gitignore

## 已知教训

### 安全
- 严禁将 token/密码/密钥写入文档或公开文件
- 敏感信息仅通过环境变量传入，`claude_workspace/` gitignore
- Web 面板当前无认证（已知问题 #34）

### LLM 行为
- prompt 保持原则式而非规则式，不要写具体例子
- 说要做的事必须输出 `<tool_call>`，不能只在回复里说"已通知"
- 更新日程用 schedule_update + id 精确匹配
- claude_code 返回结果后就完成了，不要再调一次

### Claude Code
- 使用 `--output-format stream-json --include-partial-messages --verbose`
- 超时 1800s，最多 2 个并发，配置在 `claude.json`

## 调试

| 项目 | 地址 |
|------|------|
| 日志级别 | `RUST_LOG=qq_assistant=debug` |
| Web 面板 | `http://127.0.0.1:5050` |
| LLM API | `http://127.0.0.1:8081` |
| Embed API | `http://127.0.0.1:8082` |
| NapCat HTTP | `http://127.0.0.1:4444` |
| GitHub Issues | `https://github.com/OrinVoss/Q-agent/issues` |

## Milestone 与 Issue 优先级

共 84 个 issues，分配到 3 个 Milestone：

| Milestone | 截止 | Issue 数 | 范围 |
|-----------|------|---------|------|
| **v0.2 Core** | 2026-06-14 | 11 | 安全/Panic/泄漏/SSRF |
| **v0.3 Experience** | 2026-07-14 | 39 | 稳定性/工具/Web/LLM |
| **v1.0 Stable** | 2026-08-14 | 34 | 重构/测试/硬编码 |

## 文档索引

- 完善计划：`docs/project/polish-plan.md`
- 远景规划：`docs/project/vision.md`
- 开发日志：`docs/project/dev-history.md`
- 开发者文档：`docs/technical/developer.md`
- API 参考：`docs/technical/api-reference.md`
- 依赖详情：`docs/technical/dependencies.md`
- 完整问题清单：`docs/issues/issue-list-full.md`

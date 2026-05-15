# QQ 智慧助理 — 项目规范

## 开发准则

### 修改记录（必填）
每次代码变更必须在 `changelog/` 目录下创建 Markdown 文件，格式：

```markdown
# 变更标题

**日期**: YYYY-MM-DD
**类型**: 功能新增 | Bug 修复 | 重构 | 文档 | 配置

## 变更内容
- 做了什么
- 改了哪些文件

## 影响范围
- 涉及模块
- 兼容性说明
</markdown>
```

### 代码规范
- 遵循现有 Rust 代码风格（4 空格缩进，edition 2024）
- 新增功能必须注册到 ToolRegistry
- 新增数据需添加对应的 Store 持久化
- 前端改动需同步更新 web/mod.rs API 端点

### 架构约束
- 保持只读（不发送 QQ 消息）
- 工具通过 Tool trait 注册，在 main.rs 中 build_tool_registry()
- 数据持久化统一使用 JSON 文件，放在 data/ 目录下
- LLM 调用走 agent_chat() 或 analyze()
- 系统提示词在 agent/prompt.rs 中构建

## 关键路径

| 路径 | 说明 |
|------|------|
| src/main.rs | 入口：启动 WebSocket → Agent → Web |
| src/agent/mod.rs | AgentLoop 核心编排逻辑 |
| src/tools/traits.rs | Tool trait 定义 |
| src/llm.rs | LLM 客户端（llama-server API） |
| src/store.rs | 所有持久化存储 |
| src/web/mod.rs | Web 面板 + API |
| changelog/ | 修改记录（每次提交必填） |

### 安全规范
- ⚠️ **严禁** 将 Token、密码、密钥等敏感信息写入文档、README、changelog 或任何公开文件
- 配置默认值应使用占位符（如 `your_token_here`），而非真实值
- 敏感信息仅通过环境变量传入，不硬编码在源码中
- 推送到 GitHub 前检查是否泄露了个人信息

## 调试
- 日志级别：RUST_LOG=qq_assistant=debug
- Web 面板：http://127.0.0.1:5050
- LLM API：http://127.0.0.1:8081

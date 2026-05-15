# Agent 循环重构

**日期**: 2026-05-15
**类型**: 架构重构

## 变更内容
- 删除线性流水线 processor.rs
- 实现 agent 模块：AgentLoop 编排 LLM 决策-工具调用-结果回传循环
- 实现 agent/dispatcher.rs：解析 `<tool_call>` XML 标签
- 实现 agent/prompt.rs：动态 System Prompt 构建
- 实现 Tool trait + ToolRegistry 工具注册系统 (tools/traits.rs)
- 注册 7 个工具：notify_user, qq_read, schedule, memory_write, memory_read, note_take, claude_code
- 上下文窗口管理：6144 token 阈值，自动裁剪旧消息
- 最大循环次数：10 轮

## 新增文件
- src/agent/mod.rs, src/agent/dispatcher.rs, src/agent/prompt.rs
- src/tools/traits.rs, src/tools/mod.rs
- src/tools/notify.rs, src/tools/qq_read.rs
- src/tools/schedule.rs, src/tools/memory.rs
- src/tools/note_take.rs, src/tools/claude_code.rs

## 删除文件
- src/processor.rs

# 会话历史持久化到 SQLite（修复 issue #36）

**日期**: 2026-05-16
**类型**: Bug 修复

## 变更内容
- 新增 `chat_history` SQLite 表，持久化所有会话消息
- `MessageHistoryStore` 启动时从数据库加载历史到内存
- `push()` 同时写入内存和 SQLite
- `clear()` 同时清除两者
- 构造函数改为需要 `data_dir` 参数

## 原因
- 重启后 LLM 丢失所有对话上下文，每次都是"新的对话"
- 工具调用过程也不保留

## 变更文件
- src/store.rs（MessageHistoryStore 重构，新增 chat_history 表）
- src/main.rs（传入 data_dir 参数）

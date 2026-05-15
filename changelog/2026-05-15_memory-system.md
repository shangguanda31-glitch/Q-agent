# 三层记忆系统

**日期**: 2026-05-15
**类型**: 功能新增

## 变更内容
- 实现 MemoryStore：持久化记忆文件 (data/memories.json)
- 实现 NoteStore：笔记持久化 (data/notes.json)
- Agent 循环中自动加载相关记忆到 System Prompt
- 每会话 20 条历史消息上下文
- 注册 remember/recall 工具供 LLM 主动读写
- 注册 note_take 工具供 LLM 记录笔记
- 实现会话连续上下文（chat_id 隔离）

## 新增文件
- store.rs 中 MemoryStore, NoteStore impl
- 数据文件 data/memories.json, data/notes.json

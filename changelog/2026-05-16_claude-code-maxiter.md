# 修复 Claude Code 调用失败 + 改进失败重试策略

**日期**: 2026-05-16
**类型**: Bug 修复

## 变更内容
- 修复 claude_code 工具：移除已废弃的 `--max-iter` 参数，改为 `--effort max`
- 要求 LLM 在调用 claude_code 前必须先 notify_user
- 要求 LLM 在 claude_code 连续失败 2 次后放弃重试，直接告知用户

## 原因
- 新版 Claude Code CLI 不再支持 `--max-iter`，导致工具始终返回 exit code 1
- LLM 在工具连续失败 5 次后仍然在重试，且没有发通知

## 变更文件
- src/tools/claude_code.rs（--max-iter 50 → --effort max）
- src/agent/prompt.rs（claude_code 前必须通知，失败 2 次后放弃）

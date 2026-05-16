# 简化 prompt + 修复 Claude Code 进度（心跳替代 stderr）

**日期**: 2026-05-16
**类型**: 重构

## 变更内容
- prompt 从 90 行砍到 30 行，删除死板细则，保留关键规则（通知/去重/假执行）
- claude_code 进度改用纯心跳通知（每 30s），因 claude -p --print 模式下 stderr 始终为空
- 移除无用的 stderr 读取和 ANSI 剥离代码

## 变更文件
- src/agent/prompt.rs（简化规则，保留关键点）
- src/tools/claude_code.rs（去掉 stderr 读取，改用心跳）

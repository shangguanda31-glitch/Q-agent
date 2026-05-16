# 修复 Claude Code 进度通知（ANSI 剥离 + 心跳）

**日期**: 2026-05-16
**类型**: Bug 修复

## 变更内容
- 新增 `strip_ansi()` 函数剥离 ANSI 转义码，提取进度文字
- 30 秒无进度时发送心跳通知
- 通知标题显示已运行秒数

## 原因
- claude 进度输出全是 ANSI spinner 序列，被 `contains('\x1b')` 整行过滤
- 用户收不到任何进度通知

## 变更文件
- src/tools/claude_code.rs（ANSI 剥离 + 心跳通知）

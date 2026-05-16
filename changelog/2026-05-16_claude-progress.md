# Claude Code 进度反馈 + 流式读取 stderr

**日期**: 2026-05-16
**类型**: 功能新增

## 变更内容
- claude_code 工具改为流式读取 stderr，实时提取进度信息
- 每 20 秒发一次 Windows 通知，告知当前处理状态
- 进度写入 `claude_workspace/.claude_progress` 文件
- Web 面板新增 `/api/claude-progress` 端点读取进度

## 原因
- 之前 claude_code 完全阻塞，用户不知道处理进度
- 现在通过通知和 Web 面板双重反馈

## 变更文件
- src/tools/claude_code.rs（流式读 stderr，进度通知 + 写入文件）
- src/web/mod.rs（新增 /api/claude-progress 端点）

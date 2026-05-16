# Claude Code 改用 stream-json 实时显示思考进度

**日期**: 2026-05-16
**类型**: 功能改进

## 变更内容
- claude_code 从 `--print` 模式切换为 `--output-format stream-json --include-partial-messages --verbose`
- 实时解析 JSON 流，提取 thinking 内容作为进度通知
- 检测到工具调用时通知"正在执行工具..."
- 每 10 秒推送一次思考进度

## 原因
- 之前 stderr 模式收不到进度（claude -p 模式下 stderr 为空）
- 纯心跳不展示实际进展

## 变更文件
- src/tools/claude_code.rs（stream-json 解析 + 实时进度通知）

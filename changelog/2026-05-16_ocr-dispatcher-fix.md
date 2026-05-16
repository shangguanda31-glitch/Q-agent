# 修复残留硬编码路径 + 工具调用解析日志

**日期**: 2026-05-16
**类型**: Bug 修复

## 变更内容
- ocr.rs：错误提示中的个人路径替换为通用安装指引
- dispatcher.rs：tool_call JSON 解析失败时记录 warn 日志（之前静默丢弃）

## 变更文件
- src/tools/ocr.rs（错误信息中的硬编码路径）
- src/agent/dispatcher.rs（解析失败日志 + 未闭合标签警告）

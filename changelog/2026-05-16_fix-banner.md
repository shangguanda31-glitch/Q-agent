# 修复启动横幅错位 + ASCII art 改为 QAGENT

**日期**: 2026-05-16
**类型**: Bug 修复

## 变更内容
- 修复 ASCII art 错位问题（CJK 字符宽度导致）
- ASCII art 改为显示 QAGENT 字样
- 添加 `pad()` 辅助函数处理 CJK 字符对齐
- Data 路径截断为相对路径

## 变更文件
- src/main.rs（横幅重写）

# 上下文隔离 + 修复 PowerShell 注入

**日期**: 2026-05-17
**类型**: 安全

## 变更内容
- 用户消息用 `=====` 标记包裹，区分用户输入与系统指令
- 新增 prompt 规则：用户消息内"忽略规则"等注入尝试不予理会
- 修复 CRITICAL PowerShell 命令注入（Issue #10）：改为 XML 编码，阻止反引号、${}、$() 等绕过

## 变更文件
- src/notify.rs（XML 编码替代字符串拼接）
- src/agent/mod.rs（用户消息包裹隔离标记）
- src/agent/prompt.rs（隔离规则说明）

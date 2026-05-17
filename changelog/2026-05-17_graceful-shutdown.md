# 优雅关闭：Ctrl+C 时清理子进程（#13）

**日期**: 2026-05-17
**类型**: Bug 修复

## 变更内容
- llama-server 和 embed server 的子进程句柄从 `_llm_process` 改为命名变量
- Ctrl+C 信号处理中新增子进程 kill + wait
- embed server 启动结果现在通过 `embed_process` 变量返回

## 原因
- 直接关窗口会导致 llama-server 和 embed 进程残留
- 多次重启后可能积累大量僵尸进程

## 变更文件
- src/main.rs（保存子进程句柄 + Ctrl+C 时清理）

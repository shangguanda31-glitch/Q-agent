# Claude Code 超时延长 + 执行队列

**日期**: 2026-05-16
**类型**: 功能改进

## 变更内容
- Claude Code 超时从 120 秒延长至 600 秒（10 分钟）
- 新增执行队列：同一时间只运行一个 claude_code，后续任务排队等待

## 原因
- 通过外部 API（讯飞星火）调用时响应较慢，120 秒经常超时
- 多条消息同时触发 claude_code 会启动多个进程，占用资源且互相干扰

## 变更文件
- src/config.rs（default 120→600）
- src/tools/claude_code.rs（新增 Semaphore 队列）

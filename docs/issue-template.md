# 代码质量扫描报告 - 150+问题待修复

**扫描日期**: 2026-05-16
**扫描轮次**: 50轮

## 问题统计

| 严重程度 | 数量 | 说明 |
|---------|------|------|
| CRITICAL | 1 | 命令注入漏洞 |
| HIGH | 25+ | 安全漏洞、panic 风险、资源泄漏 |
| MEDIUM | 50+ | 架构问题、错误处理不当 |
| LOW | 75+ | 代码风格、文档缺失 |

## 最紧急问题

### 安全漏洞 (CRITICAL/HIGH)
1. **PowerShell 命令注入** - `src/notify.rs:4-20` - 用户可控参数直接拼接到 PowerShell 脚本
2. **路径遍历漏洞** - `src/web/mod.rs:95-98` - 可通过 `../` 访问任意文件
3. **Token 泄露到日志** - `src/napcat/ws.rs:20`, `src/main.rs:151`
4. **Web API 无认证** - `src/web/mod.rs:23-57`

### Panic 风险 (HIGH)
- `store.rs:13` - `expect("DB open")` 数据库打开失败 panic
- `store.rs` 多处 - `prepare().unwrap()` SQL 准备失败 panic
- `store.rs:137` - `from_local_datetime().unwrap()` 时区转换失败 panic

### 资源泄漏 (HIGH)
- `main.rs:165-176` - llama-server 和 embed server 子进程句柄丢失
- `main.rs:210-260` - 异步任务未保留 JoinHandle，无法优雅关闭

## 详细报告

完整报告已保存到 `docs/code-quality-report.md`

## 优先修复建议

### 立即修复
1. PowerShell 命令注入漏洞
2. 路径遍历漏洞
3. Token 泄露到日志

### 近期修复
1. 子进程资源泄漏
2. unwrap/expect panic 风险
3. 数据库操作错误被忽略

### 中期优化
1. 架构重构
2. 统一错误处理策略
3. 添加 HTTP 请求超时配置

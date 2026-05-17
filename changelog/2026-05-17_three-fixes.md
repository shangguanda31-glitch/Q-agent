# 修复 #59 #91 #93：Banner 脱敏 + 版本号 + HTML 转义

**日期**: 2026-05-17
**类型**: Bug 修复

## 变更内容
- #59：Banner 中 NapCat WebSocket URL 截断到 `?` 前，不再显示 access_token
- #91：版本号从硬编码 `v0.1.0` 改为 `env!("CARGO_PKG_VERSION")`
- #93：`html_escape()` 补全双引号和单引号转义

## 变更文件
- src/main.rs（Banner 脱敏 + 版本号动态化）
- src/web/mod.rs（HTML 转义补全）

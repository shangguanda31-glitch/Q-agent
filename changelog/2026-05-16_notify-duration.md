# 通知持续时间延长

**日期**: 2026-05-16
**类型**: 功能改进

## 变更内容
- Windows Toast 通知增加 duration="long" 属性
- 通知显示时间从默认短时长延长为长时长
- 通知标题从 "QQ Assistant" 改为 "QAgent"

## 变更文件
- src/notify.rs（增加 duration 属性）

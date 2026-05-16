# 修复日程时间不显示

**日期**: 2026-05-16
**类型**: Bug 修复

## 变更内容
- 修复 `schedule_create` 工具中 time 参数被强行设为 None 的 bug
- 现在 LLM 传入的时间会正确保存并显示
- 更新工具描述，引导 LLM 单独传 time 参数

## 原因
第 38 行 `let time = info.as_ref().and_then(|_| None);` 导致所有日程时间永远为 null

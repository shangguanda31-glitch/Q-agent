# 新增 schedule_update 工具

**日期**: 2026-05-16
**类型**: 功能新增

## 变更内容
- 新增 `schedule_update` 工具，支持按标题匹配并追加地点等补充信息
- ScheduleStore 新增 `update_description()` 公共方法
- LLM 现在能正确处理后续补充地点的消息，更新已有日程而非创建重复

## 原因
- "在五教的315" 等后续补充消息无法更新已有日程
- 现在 LLM 收到此类消息后会调 schedule_update 追加信息

## 变更文件
- src/tools/schedule.rs（新增 ScheduleUpdateTool）
- src/store.rs（新增 update_description 方法）
- src/main.rs（注册新工具）

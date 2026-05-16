# 排除列表功能 + Web 面板管理界面

**日期**: 2026-05-16
**类型**: 功能新增

## 变更内容
- 新增排除列表系统：排除指定群或用户，Agent 自动跳过其消息
- SQLite 持久化排除配置（exclusions 表）
- NapCatApi 新增 `get_group_list()` 和 `get_friend_list()` 方法
- Web 面板新增「🚫 排除」标签页，🟢/🔴 点击切换排除状态
- 新增 API：`GET /api/exclusions`、`POST /api/exclusions/add|remove`、`GET /api/chat-sources`
- 废弃环境变量 `EXCLUDED_GROUPS`/`EXCLUDED_USERS`，统一由 Web 面板管理

## 变更文件
- src/store.rs（新增 ExclusionStore + exclusions 表）
- src/napcat/api.rs（新增 get_group_list / get_friend_list）
- src/agent/mod.rs（Agent 循环中查 ExclusionStore 跳过排除项）
- src/web/mod.rs（排除和聊天源 API 端点）
- src/web/static/index.html（排除列表 UI）
- src/config.rs（移除废弃的 excluded_groups/users 字段）
- src/main.rs（接入 ExclusionStore）

# 自动记忆读取改为语义搜索

**日期**: 2026-05-16
**类型**: 功能改进

## 变更内容
- 自动记忆上下文加载（`load_context`）现在使用 embedding 语义搜索，而非纯关键词匹配
- `load_context` 新增 `query_embedding` 参数，传入消息文本的语义向量
- 每次收到消息时自动生成 embedding 用于语义匹配

## 原因
- 纯关键词 `LIKE %text%` 无法匹配语义相关的内容（如"上次那个事"找不到相关记忆）
- 现在自动触发的记忆读取也使用余弦相似度排序，与 `recall` 工具一致

## 变更文件
- src/store.rs（load_context 新增 query_embedding 参数）
- src/agent/mod.rs（生成 embedding 后传入 load_context）

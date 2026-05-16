# 语义记忆升级

**日期**: 2026-05-16
**类型**: 功能新增

## 变更内容
- 记忆系统从关键词匹配升级为语义搜索
- 实现 `/v1/embeddings` 接口调用 (Qwen3.5-9B, 4096d)
- `remember` 工具存储内容时自动生成 embedding
- `recall` 工具搜索时用余弦相似度排序
- 关键词匹配作为降级方案（语义分数低时补足）
- 添加 `--embeddings --pooling mean` 到 llama-server 参数
- 添加 dotenvy 支持（.env 文件加载）

## 新增文件
- .env（本地配置，gitignored）
- .env.example

## 变更文件
- src/store.rs（MemoryEntry 新增 embedding 字段，read() 改为余弦相似度）
- src/tools/memory.rs（remember/recall 工具调用 LLM embed）
- src/llm.rs（新增 embed() 方法）
- src/main.rs（--embeddings --pooling mean，dotenvy）
- Cargo.toml（新增 dotenvy 依赖）

# LLM 抽象层设计

**日期**: 2026-05-18
**类型**: 架构重构

## 变更内容
将 `src/llm.rs` 重构为 `src/llm/` 模块，引入多后端抽象层：

```
src/llm/
├── mod.rs      — 模块入口，导出公共类型
├── traits.rs   — LLMProvider trait（chat + embed）
├── openai.rs   — OpenAI 兼容 API 实现（llama.cpp / OpenVINO / vLLM 共用）
└── factory.rs  — 根据配置创建对应 Provider
```

### 具体变更
- `src/llm.rs` → `src/llm/mod.rs` + 拆分
- `config.rs`: 新增 `LLMBackend` 枚举 + `llm_backend` 配置字段
- `agent/mod.rs`: `Arc<LLMClient>` → `Arc<dyn LLMProvider>`，方法名 `agent_chat` → `chat`
- `tools/memory.rs`: `Arc<LLMClient>` → `Arc<dyn LLMProvider>`
- `main.rs`: 改用 `llm::create_provider()` 工厂函数创建

### 影响范围
- 调用方代码需微调类型签名和方法名
- 功能完全不变
- 未来添加新后端只需新增 provider 文件 + factory 匹配

## 添加后端流程
1. 在 `traits.rs` 确认 trait 签名
2. 新建 `src/llm/<backend>.rs` 实现 `LLMProvider`
3. 在 `config.rs` 的 `LLMBackend` 添加枚举值
4. 在 `factory.rs` 匹配创建
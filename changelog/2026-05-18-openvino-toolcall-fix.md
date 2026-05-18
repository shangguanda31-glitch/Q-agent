# 修复 OpenVINO 后端 Tool Call 格式兼容

**日期**: 2026-05-18
**类型**: Bug 修复

## 问题
OpenVINO INT4 量化模型输出的 tool call 格式与 llama.cpp 不同：
- 输出裸 JSON 而非 `<tool_call>` XML 包裹
- 字段名使用 `"tool"` 而非 `"name"`
- 参数名使用 `"event"` 而非 `"title"` 等别名

导致 dispatcher 无法解析，LLM 工具调用全部失效。

## 修复
`src/agent/dispatcher.rs`:
1. `ParsedToolCall` 增加 `#[serde(alias = "tool")]` 兼容 `"tool"` 字段名
2. 新增裸 JSON fallback 解析：当无 XML 标签时，尝试将整个响应解析为 tool call
3. 新增 `normalize_args()` 函数：将常见参数别名映射到规范名（event→title, date→time 等）

## 影响范围
- 仅 dispatcher 模块修改，调用方无变化
- 向后兼容：XML 格式仍为首选，裸 JSON 为 fallback
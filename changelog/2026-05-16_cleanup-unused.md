# 清理 llm.rs 中未使用的旧线性流水线代码

**日期**: 2026-05-16
**类型**: 重构

## 变更内容
- 删除 `analyze()`、`analyze_with_image()`、`analyze_inner()` 三个未使用的函数
- 删除对应的 42 行 `SYSTEM_PROMPT` 常量
- 删除不再需要的 `use tracing::info` 和 `use LLMAnalysis` 导入

## 原因
- 这些是早期线性流水线时代的残留，Agent 循环中从未调用
- 留着增加认知负担和编译警告

## 变更文件
- src/llm.rs（删除 84 行无用代码）

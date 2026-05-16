# 启动横幅添加 ANSI 颜色

**日期**: 2026-05-16
**类型**: 配置

## 变更内容
- QAGENT ASCII art 和边框 → 青色
- 版本号 → 黄色
- 信息标签（NapCat/LLM/Embed/Web/Data）→ 绿色
- 使用 const 定义颜色变量 + format! 参数插入

## 变更文件
- src/main.rs（横幅添加颜色）

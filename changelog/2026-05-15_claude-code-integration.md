# Claude Code 集成

**日期**: 2026-05-15
**类型**: 功能新增

## 变更内容
- 实现 claude_code 工具：调用 Claude Code CLI 处理复杂任务
- 固定工作目录：claude_workspace/
- 添加 --max-iter 50 允许 Claude Code 多轮迭代
- 添加 --dangerously-skip-permissions 跳过权限询问
- 配置 .claude/settings.json 允许全部工具
- 工具返回包含工作区文件列表
- Claude Code 完成时发送 Windows 通知
- 通知显示执行结果摘要和创建的文件列表

## 新增文件
- tools/claude_code.rs
- claude_workspace/.claude/settings.json

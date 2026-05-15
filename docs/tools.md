# 工具参考

所有工具实现 `Tool` trait，注册在 `ToolRegistry` 中。

## Tool trait

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;           // 工具名，LLM 通过此名调用
    fn description(&self) -> &str;    // 描述，注入 System Prompt
    fn parameters_schema(&self) -> Value;  // JSON Schema
    async fn execute(&self, args: Value) -> ToolResult;
}
```

## 工具清单

### notify_user
- **功能**: Windows 桌面通知
- **参数**: `{title, body}`
- **自动调用**: 日程创建、Claude Code 完成时

### schedule_create
- **功能**: 创建日程
- **参数**: `{title, info}`
- **存储**: data/schedules.json

### schedule_list
- **功能**: 查看日程列表
- **参数**: 无

### claude_code
- **功能**: 调用 Claude Code CLI 处理复杂任务
- **参数**: `{prompt}`
- **特性**: max-iter 50, 固定工作目录 claude_workspace/
- **返回**: 执行结果 + 创建的文件列表

### ocr_image
- **功能**: Tesseract OCR 图片文字识别
- **参数**: `{image_path}`
- **语言**: 中文简体 + 英文
- **依赖**: 需安装 Tesseract 5.x

### note_take
- **功能**: 记录笔记
- **参数**: `{content, speaker, source}`
- **存储**: data/notes.json

### remember
- **功能**: 写入持久化记忆
- **参数**: `{content}`
- **存储**: data/memories.json

### recall
- **功能**: 读取记忆
- **参数**: `{query}`
- **搜索**: 内容关键词匹配

## 调用方式

LLM 通过 `<tool_call>` XML 标签调用工具：

```
<tool_call>
{"name": "tool_name", "arguments": {"param": "value"}}
</tool_call>
```

支持多工具调用（同一轮次多个 `<tool_call>` 标签按序执行）。

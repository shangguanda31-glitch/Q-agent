# 工具参考

所有工具实现 `Tool` trait，注册在 `ToolRegistry` 中。

## Tool trait

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;  // JSON Schema
    async fn execute(&self, args: Value) -> ToolResult;
}
```

## 工具清单

当前注册 9 个工具：

### notify_user
- **功能**: Windows 桌面通知弹窗
- **参数**: `{title, body}`
- **自动调用时机**: 日程创建成功、Claude Code 完成

### schedule_create
- **功能**: 保存一条日程
- **参数**: `{title, info}`（info 为自然语言描述的时间/地点等）
- **存储**: data.db (schedules 表)
- **时间解析**: 自动解析中文时间表达式（"明天下午5点"）

### schedule_list
- **功能**: 查看所有日程列表
- **参数**: 无

### claude_code
- **功能**: 调用 Claude Code CLI 处理复杂任务
- **参数**: `{prompt}`
- **特性**: max-iter 50, 固定工作目录 claude_workspace/, 跳过权限
- **通知**: 完成时推送 Windows 通知，包含创建的文件列表

### ocr_image
- **功能**: Tesseract OCR 图片文字识别
- **参数**: `{image_path}`
- **语言**: 中文简体 + 英文
- **依赖**: Tesseract 5.x（可选，未安装时返回引导提示）

### note_take
- **功能**: 记录对话中的重要笔记
- **参数**: `{content, speaker, source}`
- **存储**: data.db (notes 表)

### remember
- **功能**: 语义记忆写入
- **参数**: `{content}`
- **存储**: data.db (memories 表)，含 4096 维 embedding
- **语义搜索**: 调用 llama-server `/v1/embeddings` 生成向量

### recall
- **功能**: 语义记忆读取
- **参数**: `{query}`（自然语言描述）
- **搜索**: 余弦相似度排序 + 关键词匹配降级

### qq_read
- **功能**: 读取 QQ 群公告等信息
- **参数**: `{action, group_id?}`

## 调用方式

LLM 通过 `<tool_call>` XML 标签调用工具，支持在同一回复中调用多个工具：

```
<tool_call>
{"name": "tool_name", "arguments": {"param": "value"}}
</tool_call>
```

工具按序执行，结果通过 `<tool_result>` 回传给 LLM 进行下一轮判断。

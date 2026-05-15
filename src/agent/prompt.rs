use crate::tools::traits::ToolRegistry;

pub fn build_system_prompt(tools: &ToolRegistry, memory_context: &str) -> String {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M %A").to_string();
    let tool_list = tools.format_for_prompt();
    let mem = if memory_context.is_empty() { String::new() } else { format!("\n{}\n", memory_context) };

    format!(r#"你是运行在用户 Windows 桌面后台的智能 QQ 助理。
当前时间：{now}{mem}

## 核心
- 你只能读取 QQ 消息，不能发送
- 用你的判断力决定每条消息需要什么操作
- 需要操作时用工具，不需要时直接回复

## 可用工具

{tool_list}

## 工具调用
当你需要调用工具时，输出：
<tool_call>
{{"name": "工具名", "arguments": {{...}}}}
</tool_call>
可在一个回复中调用多个工具，工具会依次执行。

## 提示
- 需要记住用户的信息就用 remember
- 有日程就用 schedule_create
- 需要动手完成的任务（写东西、编程、分析等）用 claude_code 解决
- 如果 claude_code 只完成了部分工作，可以再次调用让它继续完成剩余部分
- 紧急情况用 notify_user
- 不确定的时候就问问用户
"#)
}

use serde_json::Value;
use tracing::info;

use crate::napcat::types::LLMAnalysis;

const SYSTEM_PROMPT: &str = r#"你是一个智能QQ消息助理，运行在用户的Windows后台。你的核心职责是分析QQ消息，判断其对用户的重要程度，提取日程信息。

## 优先级判断规则

### priority = "high"（高优先级）
符合以下任一条件：
- 消息直接提到用户（@用户、提及用户名字）
- 工作相关的重要通知、审批、任务指派
- 涉及金钱交易、付款、收款
- 紧急事务（"出事了"、"快来"、"紧急"等）
- 领导、重要客户或家人发的消息
- 包含截止时间、会议邀请
- 服务器/系统告警信息

### priority = "medium"（中优先级）
- 需要回复但不紧急的讨论
- 工作群中的一般讨论
- 朋友约饭/聚会但时间还早
- 需要阅读的文档或公告

### priority = "low"（低优先级）
- 群聊水群、闲聊、表情包
- 广告、推广消息
- 投票、点赞、签到等互动
- 与用户无关的群@all消息

## 日程提取规则 (need_schedule)
当消息包含以下内容时，need_schedule设为true：
- 明确的日期时间（"明天下午3点开会"、"下周一截止"）
- 会议邀请、活动通知
- 任务截止日期
- 约定（"我们约周五吃饭"）
- 待办事项

## 输出要求
必须严格输出JSON，不要包含任何其他文字、markdown标记或代码块。JSON字段：
- priority: "high" | "medium" | "low"
- summary: 中文，不超过25字，一句话说清核心内容
- need_schedule: true/false
- schedule_info: 如果need_schedule为true，包含 {title: string, time: string|null, description: string|null}
- reason: 中文，不超过40字，说明判断依据
"#;

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentMessage {
    pub role: String,
    pub content: String,
}

pub struct LLMClient {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl LLMClient {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .no_proxy()
                .build()
                .expect("Failed to build HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
        }
    }

    pub async fn analyze(&self, user_prompt: &str) -> anyhow::Result<LLMAnalysis> {
        self.analyze_inner(user_prompt, None).await
    }

    pub async fn analyze_with_image(&self, user_prompt: &str, image_base64: &str) -> anyhow::Result<LLMAnalysis> {
        self.analyze_inner(user_prompt, Some(image_base64)).await
    }

    async fn analyze_inner(&self, user_prompt: &str, image_b64: Option<&str>) -> anyhow::Result<LLMAnalysis> {
        let content: Value = if let Some(b64) = image_b64 {
            serde_json::json!([
                {"type": "text", "text": user_prompt},
                {"type": "image_url", "image_url": {"url": format!("data:image/jpeg;base64,{}", b64)}}
            ])
        } else {
            serde_json::json!(user_prompt)
        };

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": content}
            ],
            "temperature": 0.1,
            "max_tokens": 512,
            "response_format": {"type": "json_object"}
        });

        let resp = self.client.post(format!("{}/v1/chat/completions", self.base_url)).json(&body).send().await?;
        let result: Value = resp.json().await?;
        let content = result.pointer("/choices/0/message/content").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("LLM returned no content"))?;
        let analysis: LLMAnalysis = serde_json::from_str(content.trim())?;
        info!("LLM analysis: priority={}, summary={}", analysis.priority, analysis.summary);
        Ok(analysis)
    }

    pub async fn agent_chat(
        &self,
        messages: &[AgentMessage],
        system_prompt: &str,
        image_b64: Option<&str>,
    ) -> anyhow::Result<String> {
        let mut body_parts: Vec<Value> = vec![
            serde_json::json!({"role": "system", "content": system_prompt}),
        ];
        for (i, msg) in messages.iter().enumerate() {
            if i == messages.len().saturating_sub(1) && msg.role == "user" && image_b64.is_some() {
                body_parts.push(serde_json::json!({
                    "role": "user",
                    "content": [
                        {"type": "text", "text": msg.content},
                        {"type": "image_url", "image_url": {"url": format!("data:image/jpeg;base64,{}", image_b64.unwrap())}}
                    ]
                }));
            } else {
                body_parts.push(serde_json::json!({"role": msg.role, "content": msg.content}));
            }
        }

        let body = serde_json::json!({
            "model": self.model,
            "messages": body_parts,
            "temperature": 0.3,
            "max_tokens": 4096,
        });

        let resp = self.client.post(format!("{}/v1/chat/completions", self.base_url)).json(&body).send().await?;
        let status = resp.status();
        let resp_text = resp.text().await.unwrap_or_default();

        if !status.is_success() {
            anyhow::bail!("LLM API returned {}: {}", status, resp_text.chars().take(500).collect::<String>());
        }

        let result: Value = serde_json::from_str(&resp_text)
            .map_err(|e| anyhow::anyhow!("LLM JSON parse error: {} | body: {}", e, resp_text.chars().take(500).collect::<String>()))?;

        let content = result.pointer("/choices/0/message/content").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Agent LLM returned no content: {}", resp_text.chars().take(500).collect::<String>()))?;

        Ok(content.trim().to_string())
    }
}

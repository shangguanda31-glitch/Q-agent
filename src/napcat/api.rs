use reqwest::Client;
use serde_json::Value;
use tracing::warn;

pub struct NapCatApi {
    client: Client,
    base_url: String,
    token: String,
}

impl NapCatApi {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            client: Client::builder()
                .no_proxy()
                .build()
                .expect("Failed to build HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    pub async fn get_image_info(&self, file: &str) -> Option<String> {
        let resp = self
            .client
            .post(format!("{}/get_image", self.base_url))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({ "file": file }))
            .send()
            .await
            .ok()?;

        let body: Value = resp.json().await.ok()?;
        body.pointer("/data/url").and_then(|v| v.as_str().map(String::from))
    }

    pub async fn get_group_notices(&self, group_id: i64) -> Vec<Value> {
        let resp = self
            .client
            .post(format!("{}/get_group_notice", self.base_url))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({ "group_id": group_id }))
            .send()
            .await;

        match resp {
            Ok(r) => {
                match r.json::<Value>().await {
                    Ok(body) => {
                        body.pointer("/data/notices")
                            .and_then(|v| v.as_array())
                            .cloned()
                            .unwrap_or_default()
                    }
                    Err(e) => {
                        warn!("Failed to parse group notices response: {}", e);
                        vec![]
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch group notices: {}", e);
                vec![]
            }
        }
    }

    pub async fn get_group_file_url(&self, group_id: i64, file_id: &str, bus_id: i32) -> Option<String> {
        let resp = self
            .client
            .post(format!("{}/get_group_file_url", self.base_url))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({
                "group_id": group_id,
                "file_id": file_id,
                "bus_id": bus_id,
            }))
            .send()
            .await
            .ok()?;

        let body: Value = resp.json().await.ok()?;
        body.pointer("/data/url").and_then(|v| v.as_str().map(String::from))
    }

    pub async fn download_file(&self, url: &str) -> Option<Vec<u8>> {
        self.client
            .get(url)
            .send()
            .await
            .ok()?
            .bytes()
            .await
            .ok()
            .map(|b| b.to_vec())
    }

    pub async fn get_group_list(&self) -> Vec<Value> {
        let resp = self.client
            .post(format!("{}/get_group_list", self.base_url))
            .header("Authorization", self.auth_header())
            .send()
            .await;
        match resp {
            Ok(r) => match r.json::<Value>().await {
                Ok(v) => v.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default(),
                Err(_) => vec![],
            },
            Err(_) => vec![],
        }
    }

    pub async fn get_friend_list(&self) -> Vec<Value> {
        let resp = self.client
            .post(format!("{}/get_friend_list", self.base_url))
            .header("Authorization", self.auth_header())
            .send()
            .await;
        match resp {
            Ok(r) => match r.json::<Value>().await {
                Ok(v) => v.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default(),
                Err(_) => vec![],
            },
            Err(_) => vec![],
        }
    }

    pub async fn get_group_msg_history(&self, group_id: i64, count: i32) -> Vec<Value> {
        let resp = self.client
            .post(format!("{}/get_group_msg_history", self.base_url))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({"group_id": group_id.to_string(), "count": count, "reverse_order": false, "disable_get_url": false, "parse_mult_msg": false, "quick_reply": false}))
            .send()
            .await;
        match resp {
            Ok(r) => match r.json::<Value>().await {
                Ok(v) => v.pointer("/data/messages").and_then(|d| d.as_array()).cloned().unwrap_or_default(),
                Err(_) => vec![],
            },
            Err(_) => vec![],
        }
    }

    pub async fn get_friend_msg_history(&self, user_id: i64, count: i32) -> Vec<Value> {
        let resp = self.client
            .post(format!("{}/get_friend_msg_history", self.base_url))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({"user_id": user_id.to_string(), "count": count, "reverse_order": false, "disable_get_url": false, "parse_mult_msg": false, "quick_reply": false}))
            .send()
            .await;
        match resp {
            Ok(r) => match r.json::<Value>().await {
                Ok(v) => v.pointer("/data/messages").and_then(|d| d.as_array()).cloned().unwrap_or_default(),
                Err(_) => vec![],
            },
            Err(_) => vec![],
        }
    }
}
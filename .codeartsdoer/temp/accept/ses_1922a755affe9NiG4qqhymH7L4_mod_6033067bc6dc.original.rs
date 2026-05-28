use reqwest::Client;
use serde_json::json;
use std::env;
use std::time::Duration;
use anyhow::{Result, anyhow};
use super::session::message::Message;

pub async fn chat(messages: &[Message]) -> Result<String> {
    let api_key = env::var("DEEPSEEK_KEY").map_err(|_| anyhow!("未设置 DEEPSEEK_KEY 环境变量，请在 .env 文件中配置"))?;

    // 自定义客户端
    let client = Client::builder().timeout(Duration::from_secs(60)).build()?;

    // 发送请求
    let res = client
        .post("https://api.deepseek.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "deepseek-chat",
            "messages": messages
        }))
        .send()
        .await?;

    // 返回的状态
    let status = res.status();

     // 解析结果
    let text = res.text().await?;

    if !status.is_success() {
        return Err(
            anyhow!(
                "API 请求失败 (HTTP {}): {}",
                status,
                text
            )
        );
    }

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|_| anyhow!("API 响应 JSON 解析失败: {}", text))?;

    let content = json["choices"][0]
        ["message"]["content"]
        .as_str()
        .ok_or_else(|| {
            anyhow!("API 响应格式异常，无法提取回复内容")
        })?;

    Ok(content.to_string())
}

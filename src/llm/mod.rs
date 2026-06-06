use reqwest::Client;
use serde_json::json;
use std::{ env, time::Duration };
use dotenvy::dotenv;
use anyhow::{Result, anyhow};
use futures::{StreamExt, stream::BoxStream};
use eventsource_stream::Eventsource;
use super::session::message::Message;

pub async fn chat(messages: &[Message]) -> Result<BoxStream<'static, Result<String>>> {
    // 加载环境变量
    dotenv().ok();

    // 获取环境变量
    let api_key = env::var("DEEPSEEK_KEY")?;

    // 自定义客户端
    let client = Client::builder().timeout(Duration::from_secs(60)).build()?;

    // 发送请求
    let res = client
        .post("https://api.deepseek.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "deepseek-v4-flash",
            "messages": messages,
            "stream": true
        }))
        .send()
        .await?;

    // 先保存状态码
    let status = res.status();

    if !status.is_success() {
        let text = res.text().await?;
        return Err(anyhow!("HTTP {}: {}", status, text));
    }

     // 解析结果
    let stream = res
        .bytes_stream()
        .eventsource()
        .map(|event| -> Result<String> {
            let event = event.map_err(|e| anyhow!("SSE 错误: {}", e))?;
            let data = event.data;

            if data.trim() == "[DONE]" {
                return Ok(String::new());
            }

            let json: serde_json::Value = serde_json::from_str(&data)?;
            let content = json["choices"][0]["delta"]["content"]
                .as_str()
                .unwrap_or("");

            Ok(content.to_string())
        })
        .boxed();

    Ok(stream)
}

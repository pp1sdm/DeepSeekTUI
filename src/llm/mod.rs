use reqwest::Client;
use serde_json::json;
use std::{ env, time::Duration };
use dotenvy::dotenv;
use anyhow::{Result, anyhow};
use super::message::Message;

pub async fn chat(messages: &[Message]) -> Result<serde_json::Value> {
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
            "messages": messages
        }))
        .send()
        .await?;
    
    // 返回的状态
    let status = res.status();
    
     // 解析结果
    let text = res.text().await?;
    
    // 文本序列化
    let json: serde_json::Value = serde_json::from_str(&text)?;
    
    // 返回
    if !status.is_success() {
        return Err(anyhow!("HTTP {}: {}", status, text));
    }

    Ok(json)
}
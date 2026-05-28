use std::env;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let api_key = env::var("DEEPSEEK_KEY").expect("DEEPSEEK_KEY not set");
    println!("API Key: {}...{}", &api_key[..8], &api_key[api_key.len()-4..]);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("failed to build client");

    println!("Sending request to DeepSeek API...");

    let res = client
        .post("https://api.deepseek.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "deepseek-chat",
            "messages": [{"role": "user", "content": "hello"}]
        }))
        .send()
        .await;

    match res {
        Ok(response) => {
            let status = response.status();
            println!("Status: {}", status);
            let text = response.text().await.unwrap();
            println!("Response: {}", &text[..text.len().min(500)]);
        }
        Err(e) => {
            println!("Request failed: {:?}", e);
            if e.is_connect() {
                println!(" -> Connection error (HTTPS/TLS issue?)");
            }
            if e.is_timeout() {
                println!(" -> Timeout error");
            }
            if e.is_request() {
                println!(" -> Request construction error");
            }
        }
    }
}

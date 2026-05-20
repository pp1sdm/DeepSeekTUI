use serde_json;

mod llm;

mod message;
mod session;

#[tokio::main]
async fn main() {
    let res = llm::chat("你的名字是什么").await;
    match res {
        Ok(text) => {
            let json = serde_json::json!({
                "result": text
            });

            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        Err(e) => {
            let json = serde_json::json!({
                "error": e.to_string()
            });

            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
    }
}

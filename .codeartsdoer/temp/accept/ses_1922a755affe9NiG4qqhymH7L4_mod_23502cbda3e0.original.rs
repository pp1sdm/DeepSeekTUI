use super::llm::chat;
use super::session::message::Message;
use anyhow::Result;

pub struct Agent;

impl Agent {
    pub async fn run(&self, messages: &[Message]) -> Result<String> {
        tracing::info!("正在进入 agent::run");
        let reply = chat(messages).await?;
        Ok(reply)
    }
}

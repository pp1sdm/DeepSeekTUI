use super::llm::chat;
use super::session::message::Message;
pub struct Agent;

impl Agent {
    pub async fn run(&self, messages: &[Message]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let reply =
            chat(messages).await?;

        Ok(reply)
    }
}
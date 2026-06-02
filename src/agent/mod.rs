use super::llm::chat;
use futures::stream::BoxStream;
use super::session::message::Message;

pub struct Agent;

impl Agent {
    pub async fn run(
        &self,
        messages: &[Message],
    ) -> anyhow::Result<BoxStream<'static, anyhow::Result<String>>> {
    let stream = chat(messages).await?;
    Ok(stream)
    }
}
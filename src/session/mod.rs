pub mod message;

use self::message::Message;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub messages: Vec<Message>,
    pub created_at: u64,
    pub updated_at: u64,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl Session {
    pub fn new() -> Self {
        let now = now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            messages: vec![],
            created_at: now,
            updated_at: now,
        }
    }

    // 流式的起点，空字符串实现占位
    pub fn start_user(&mut self) {
        self.messages.push(Message::user(String::new()));
    }

    pub fn start_assistant(&mut self) {
        self.messages.push(Message::assistant(String::new()));
    }

    pub fn start_system(&mut self) {
        self.messages.push(Message::system(String::new()));
    }

    //向流式的信息中添加数据
    pub fn append_to_last(&mut self, chunk: &str) {
        if let Some(last) = self.messages.last_mut() {
            last.content.push_str(chunk);
        }
    }

    // 收尾，补充完整的message信息的时间戳
    pub fn finish(&mut self) {
        self.updated_at = now();
    }

    // 辅助方法

    pub fn clear(&mut self) {
        self.messages.clear();
        self.updated_at = now();
    }

    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }
}
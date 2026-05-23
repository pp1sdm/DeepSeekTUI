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

    pub fn add_user_message(&mut self, msg: String) {
        // 实现message类型
        self.messages.push(
            Message::user(msg)
        );
        self.updated_at = now();
    }

    pub fn add_assistant_message(&mut self, msg: String) {
        self.messages.push(
            Message::assistant(msg)
        );
        self.updated_at = now();
    }

    pub fn add_system_message(&mut self, msg: String) {
        self.messages.push(
            Message::system(msg)
        );
        self.updated_at = now();
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.updated_at = now();
    }

    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }
}
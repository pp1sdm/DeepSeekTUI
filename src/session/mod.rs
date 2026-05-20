use serde::{Serialize, Deserialize};
use chrono::Local;

use crate::message::Message;

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub id: String,

    // 对话历史
    pub messages: Vec<Message>,

    // 元信息
    pub title: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,

    // 可选：状态
    pub is_streaming: bool,

    // 可选：模型配置
    pub model: String,
    pub temperature: f32,
}

impl Session {
    pub fn new(id: String, model: String) -> Self {
        Self {
            id,
            messages: vec![],
            title: None,
            created_at: Local::now().timestamp_millis() as u64,
            updated_at: Local::now().timestamp_millis() as u64,
            is_streaming: false,
            model,
            temperature: 0.6,
        }
    }
}
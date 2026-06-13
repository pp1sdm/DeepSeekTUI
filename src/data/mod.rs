use sqlx::SqlitePool;

/// 数据库实例
pub struct MemoryDB {
    pub pool: SqlitePool,
}

/// 原始聊天记录
#[derive(Debug)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
}

/// 长期记忆
#[derive(Debug)]
pub struct Memory {
    pub id: String,

    pub content: String,

    pub importance: f32,

    pub created_at: i64,

    pub expires_at: Option<i64>,

    pub source_message_id: Option<String>,
}

/// 向量数据
#[derive(Debug)]
pub struct MemoryEmbedding {
    pub memory_id: String,

    pub model: String,

    pub dimensions: i32,

    pub embedding: Vec<u8>,
}
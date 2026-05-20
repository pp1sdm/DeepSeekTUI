use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn role(&self) -> &String {
        &self.role
    }

    pub fn content(&self) -> &String {
        &self.content
    }
    pub fn new(role: String, content: String) -> Self {
        Self { role, content }
    }
    
    
    
}
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    System,
    User,
    Assistant,
    Function,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::Function => write!(f, "function"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

impl Message {
    pub fn new(role: Role, content: &str) -> Self {
        Self {
            role,
            content: content.to_string(),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Conversation {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            messages: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn add_message(&mut self, role: Role, content: &str) {
        self.messages.push(Message::new(role, content));
        self.updated_at = Utc::now();
    }

    pub fn add_user_message(&mut self, content: &str) {
        self.add_message(Role::User, content);
    }

    pub fn add_system_message(&mut self, content: &str) {
        self.add_message(Role::System, content);
    }

    pub fn add_assistant_message(&mut self, content: &str) {
        self.add_message(Role::Assistant, content);
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let dir = path.parent().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        fs::create_dir_all(dir)?;
        
        let serialized = serde_json::to_string_pretty(self)?;
        fs::write(path, serialized)?;
        
        Ok(())
    }

    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let conversation: Conversation = serde_json::from_str(&content)?;
        Ok(conversation)
    }
} 
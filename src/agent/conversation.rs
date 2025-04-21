use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;
use fs_err as fs;
use std::io::{self, Write};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: Role,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Message {
    pub fn new(role: Role, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            created_at: Utc::now(),
        }
    }

    pub fn user(content: String) -> Self {
        Self::new(Role::User, content)
    }

    pub fn assistant(content: String) -> Self {
        Self::new(Role::Assistant, content)
    }

    pub fn system(content: String) -> Self {
        Self::new(Role::System, content)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Conversation {
    pub fn new(title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let json = serde_json::to_string_pretty(self)?;
        let mut file = fs::File::create(path)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let json = fs::read_to_string(path)?;
        let conversation: Conversation = serde_json::from_str(&json)?;
        
        Ok(conversation)
    }

    pub fn to_openai_messages(&self) -> Vec<serde_json::Value> {
        self.messages
            .iter()
            .map(|msg| {
                serde_json::json!({
                    "role": match msg.role {
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::System => "system",
                    },
                    "content": msg.content
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationList {
    pub conversations: Vec<ConversationSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
}

impl From<&Conversation> for ConversationSummary {
    fn from(conversation: &Conversation) -> Self {
        Self {
            id: conversation.id.clone(),
            title: conversation.title.clone(),
            created_at: conversation.created_at,
            updated_at: conversation.updated_at,
            message_count: conversation.messages.len(),
        }
    }
}

impl ConversationList {
    pub fn new() -> Self {
        Self {
            conversations: Vec::new(),
        }
    }

    pub fn add_conversation(&mut self, conversation: &Conversation) {
        let summary = ConversationSummary::from(conversation);
        
        // Remove any existing entry with the same ID
        self.conversations.retain(|c| c.id != summary.id);
        
        // Add the new summary
        self.conversations.push(summary);
        
        // Sort by updated_at (most recent first)
        self.conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let json = serde_json::to_string_pretty(self)?;
        let mut file = fs::File::create(path)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        match fs::read_to_string(path) {
            Ok(json) => {
                let list: ConversationList = serde_json::from_str(&json)?;
                Ok(list)
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                Ok(ConversationList::new())
            }
            Err(err) => Err(err.into()),
        }
    }
} 
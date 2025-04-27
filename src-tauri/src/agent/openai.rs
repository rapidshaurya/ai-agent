use crate::config::Config;
use super::conversation::Conversation;
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatResponseChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatResponseChoice>,
}

pub struct OpenAIAgent {
    config: Config,
    client: Client,
    current_conversation_id: Option<String>,
}

impl OpenAIAgent {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            config,
            client: Client::new(),
            current_conversation_id: None,
        })
    }

    pub async fn send_message(&self, conversation: &mut Conversation, _message: &str) -> Result<String> {
        // Convert conversation messages to the format expected by the API
        let messages: Vec<ChatMessage> = conversation.messages.iter().map(|msg| {
            ChatMessage {
                role: msg.role.to_string(),
                content: msg.content.clone(),
            }
        }).collect();

        // Prepare the request
        let request = ChatRequest {
            model: self.config.openai_api_model.clone(),
            messages,
        };

        // Send the request to the API
        let response = self.client
            .post(format!("{}/chat/completions", self.config.openai_api_base_url))
            .header("Authorization", format!("Bearer {}", self.config.openai_api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        // Check for errors
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("API error: {}", error_text));
        }

        // Parse the response
        let chat_response: ChatResponse = response.json().await?;
        let response_message = chat_response
            .choices
            .first()
            .ok_or_else(|| anyhow!("No response from API"))?
            .message
            .content
            .clone();

        // Add the response to the conversation
        conversation.add_assistant_message(&response_message);
        
        // Save the conversation
        let path = self.get_conversation_path(&conversation.id)?;
        conversation.save(&path)?;

        Ok(response_message)
    }
    
    fn get_conversation_path(&self, id: &str) -> Result<PathBuf> {
        let mut path = self.config.history_path.clone();
        path.push(format!("{}.json", id));
        Ok(path)
    }
    
    pub async fn get_or_create_conversation(&self) -> Result<Conversation> {
        if let Some(id) = &self.current_conversation_id {
            let path = self.get_conversation_path(id)?;
            if path.exists() {
                return Conversation::load(&path);
            }
        }
        
        let mut conversation = Conversation::new();
        
        // Add a system message to start the conversation
        conversation.add_system_message(&format!(
            "You are {}, a helpful AI assistant.",
            self.config.agent_name
        ));
        
        // Save the new conversation
        let path = self.get_conversation_path(&conversation.id)?;
        conversation.save(&path)?;
        
        Ok(conversation)
    }
    
    pub async fn list_conversations(&self) -> Result<Vec<String>> {
        if !self.config.history_path.exists() {
            return Ok(Vec::new());
        }
        
        let entries = fs::read_dir(&self.config.history_path)?;
        let mut conversation_ids = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Some(stem) = path.file_stem() {
                    if let Some(id) = stem.to_str() {
                        // Validate that it's a UUID format
                        if Uuid::parse_str(id).is_ok() {
                            conversation_ids.push(id.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(conversation_ids)
    }
    
    pub async fn load_conversation(&self, id: &str) -> Result<Conversation> {
        let path = self.get_conversation_path(id)?;
        Conversation::load(&path)
    }
} 
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod agent;

use std::sync::{Arc, Mutex};
use tauri::State;
use serde::{Deserialize, Serialize};
use tracing_subscriber::prelude::*;

// Agent state that will be shared with the frontend
struct AgentState {
    config: config::Config,
    agent: Mutex<Option<Arc<agent::OpenAIAgent>>>,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct ChatSettings {
    openai_api_key: String,
    openai_api_base_url: String,
    openai_api_model: String,
    history_path: String,
}

// Commands for the frontend to interact with the agent
#[tauri::command]
async fn initialize_agent(
    state: State<'_, AgentState>,
    settings: ChatSettings,
) -> Result<bool, String> {
    let mut config = state.config.clone();
    
    config.openai_api_key = settings.openai_api_key;
    config.openai_api_base_url = settings.openai_api_base_url;
    config.openai_api_model = settings.openai_api_model;
    
    if !settings.history_path.is_empty() {
        // Convert ~ to home directory if present
        let path = settings.history_path.replace(
            "~", 
            dirs::home_dir().unwrap_or_default().to_str().unwrap_or("")
        );
        config.history_path = std::path::PathBuf::from(path);
    }
    
    // Create directory for history if it doesn't exist
    if let Some(parent) = config.history_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    let agent = agent::OpenAIAgent::new(config.clone())
        .map_err(|e| e.to_string())?;
    
    // Update the agent state
    *state.agent.lock().unwrap() = Some(Arc::new(agent));
    
    Ok(true)
}

#[tauri::command]
async fn send_message(
    state: State<'_, AgentState>,
    message: String,
) -> Result<Vec<ChatMessage>, String> {
    // Clone the Arc to avoid holding the MutexGuard across await points
    let agent = {
        let agent_lock = state.agent.lock().unwrap();
        agent_lock.clone().ok_or("Agent not initialized")?
    };
    
    // Create a conversation if it doesn't exist or get the existing one
    let mut conversation = agent.get_or_create_conversation().await
        .map_err(|e| e.to_string())?;
    
    // Add user message
    conversation.add_user_message(&message);
    
    // Send message to AI and get response
    let _response = agent.send_message(&mut conversation, &message)
        .await
        .map_err(|e| e.to_string())?;
    
    // Convert conversation messages to ChatMessage format
    let messages = conversation.messages.iter().map(|msg| {
        ChatMessage {
            role: msg.role.to_string(),
            content: msg.content.clone(),
        }
    }).collect();
    
    Ok(messages)
}

#[tauri::command]
async fn get_conversation_history(
    state: State<'_, AgentState>,
) -> Result<Vec<ChatMessage>, String> {
    // Clone the Arc to avoid holding the MutexGuard across await points
    let agent = {
        let agent_lock = state.agent.lock().unwrap();
        agent_lock.clone().ok_or("Agent not initialized")?
    };
    
    let conversation = agent.get_or_create_conversation().await
        .map_err(|e| e.to_string())?;
    
    // Convert conversation messages to ChatMessage format
    let messages = conversation.messages.iter().map(|msg| {
        ChatMessage {
            role: msg.role.to_string(),
            content: msg.content.clone(),
        }
    }).collect();
    
    Ok(messages)
}

#[tauri::command]
fn get_default_settings() -> ChatSettings {
    ChatSettings {
        openai_api_key: String::new(),
        openai_api_base_url: "https://api.openai.com/v1".to_string(),
        openai_api_model: "gpt-4-turbo".to_string(),
        history_path: dirs::home_dir()
            .unwrap_or_default()
            .join(".ai-agent")
            .join("history")
            .to_str()
            .unwrap_or("")
            .to_string(),
    }
}

#[tauri::command]
async fn select_folder(title: String) -> Result<String, String> {
    let folder = tauri::api::dialog::blocking::FileDialogBuilder::new()
        .set_title(&title)
        .pick_folder()
        .ok_or_else(|| "No folder selected".to_string())?;
    
    Ok(folder.to_string_lossy().to_string())
}

fn main() {
    // Set up tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    // Load configuration
    let config = config::Config::load().unwrap_or_default();
    
    // Create initial agent state
    let agent_state = AgentState {
        config: config.clone(),
        agent: Mutex::new(None),
    };
    
    tauri::Builder::default()
        .manage(agent_state)
        .invoke_handler(tauri::generate_handler![
            initialize_agent,
            send_message,
            get_conversation_history,
            get_default_settings,
            select_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

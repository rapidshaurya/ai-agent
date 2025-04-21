use serde::{Deserialize, Serialize};
use std::env;
use anyhow::Result;
use dotenv::dotenv;
use dirs::home_dir;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServers {
    pub context7: McpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub openai_api_key: String,
    pub openai_api_base_url: String,
    pub openai_api_model: String,
    pub agent_name: String,
    pub history_path: PathBuf,
    pub mcp_servers: McpServers,
}

impl Default for Config {
    fn default() -> Self {
        let mut history_path = home_dir().unwrap_or_else(|| PathBuf::from("."));
        history_path.push(".ai-agent");
        history_path.push("history");

        Self {
            openai_api_key: String::new(),
            openai_api_base_url: "https://api.openai.com/v1".to_string(),
            openai_api_model: "gpt-4-turbo".to_string(),
            agent_name: "ai-assistant".to_string(),
            history_path,
            mcp_servers: McpServers {
                context7: McpConfig {
                    command: "npx".to_string(),
                    args: vec!["-y".to_string(), "@upstash/context7-mcp@latest".to_string()],
                },
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load environment variables from .env file
        dotenv().ok();
        
        // Start with default configuration
        let mut config = Config::default();
        
        // Override with environment variables if they exist
        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            config.openai_api_key = api_key;
        }
        
        if let Ok(api_base) = env::var("OPENAI_API_BASE_URL") {
            config.openai_api_base_url = api_base;
        }
        
        if let Ok(api_model) = env::var("OPENAI_API_MODEL") {
            config.openai_api_model = api_model;
        }
        
        if let Ok(agent_name) = env::var("AGENT_NAME") {
            config.agent_name = agent_name;
        }
        
        if let Ok(history_path) = env::var("HISTORY_PATH") {
            let path = history_path.replace("~", home_dir().unwrap_or_default().to_str().unwrap_or(""));
            config.history_path = PathBuf::from(path);
        }
        
        // Validate required configuration
        if config.openai_api_key.is_empty() {
            anyhow::bail!("OPENAI_API_KEY environment variable is required");
        }
        
        Ok(config)
    }
} 
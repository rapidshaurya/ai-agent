use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::config::Config;
use crate::mcp;
use super::conversation::{Conversation, Message};

#[derive(Clone, Debug)]
pub struct OpenAIAgent {
    config: Config,
    client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionResponse {
    id: Option<String>,
    object: Option<String>,
    created: Option<u64>,
    model: Option<String>,
    choices: Vec<ChatCompletionChoice>,
    usage: Option<ChatCompletionUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionChoice {
    index: Option<u32>,
    message: ChatCompletionMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionMessage {
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: FunctionCall,
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl OpenAIAgent {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn chat(&self, conversation: &Conversation) -> Result<Message> {
        // Ensure MCP server is running - but continue if it fails
        let mcp_server_available = mcp::ensure_mcp_server_running(&self.config).await.is_ok();
        
        // Determine if we're using OpenAI, Ollama, Groq, or another provider
        let is_ollama = self.config.openai_api_base_url.contains("ollama") ||
                       self.config.openai_api_base_url.contains("localhost");
        let is_groq = self.config.openai_api_base_url.contains("groq");
        
        // Create the request to API
        let request = ChatCompletionRequest {
            model: self.config.openai_api_model.clone(),
            messages: conversation.to_openai_messages(),
            temperature: if is_ollama { None } else { Some(0.7) },
            stream: if is_ollama { None } else { Some(false) },
            tools: if is_ollama || is_groq || !mcp_server_available { None } else { Some(self.get_tools()) },
        };
        
        debug!("Sending chat completion request to API: {:?}", request);
        
        // Make the API request
        let url = format!("{}/chat/completions", self.config.openai_api_base_url);
        let mut req_builder = self.client.post(&url)
            .header("Content-Type", "application/json");
            
        // Add authorization header unless we're using Ollama (which doesn't need it)
        if !is_ollama {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", self.config.openai_api_key));
        }
        
        let response = req_builder
            .json(&request)
            .send()
            .await?;
        
        // Handle the response
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("API error: {} - {}", status, error_text));
        }
        
        let response_json: ChatCompletionResponse = response.json().await?;
        debug!("Received chat completion response: {:?}", response_json);
        
        // Process the response
        if let Some(choice) = response_json.choices.first() {
            let content = if let Some(tool_calls) = &choice.message.tool_calls {
                // Handle tool calls
                let mut result = String::new();
                
                for tool_call in tool_calls {
                    if tool_call.call_type == "function" {
                        let function_name = &tool_call.function.name;
                        let arguments: Value = serde_json::from_str(&tool_call.function.arguments)?;
                        
                        match function_name.as_str() {
                            "mcp_context7_resolve_library_id" => {
                                if let Some(library_name) = arguments.get("libraryName").and_then(|v| v.as_str()) {
                                    info!("Resolving library ID for: {}", library_name);
                                    match mcp::resolve_library_id(library_name.to_string()).await {
                                        Ok(library_id) => {
                                            result.push_str(&format!("Library ID for '{}' is: {}\n", library_name, library_id));
                                        },
                                        Err(e) => {
                                            result.push_str(&format!("Failed to resolve library ID for '{}': {}\n", library_name, e));
                                        }
                                    }
                                }
                            },
                            "mcp_context7_get_library_docs" => {
                                if let Some(library_id) = arguments.get("context7CompatibleLibraryID").and_then(|v| v.as_str()) {
                                    let tokens = arguments.get("tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                                    let topic = arguments.get("topic").and_then(|v| v.as_str()).map(|v| v.to_string());
                                    
                                    info!("Getting library docs for: {}", library_id);
                                    match mcp::get_library_docs(library_id.to_string(), tokens, topic).await {
                                        Ok(docs) => {
                                            // Truncate if too long for readability
                                            let docs_preview = if docs.len() > 500 {
                                                format!("{}... (truncated, {} total characters)", &docs[..500], docs.len())
                                            } else {
                                                docs.clone()
                                            };
                                            
                                            result.push_str(&format!("Documentation for '{}':\n{}\n", library_id, docs_preview));
                                            
                                            // Actually add the full documentation
                                            let full_response = format!("Based on the documentation for '{}':\n\n{}", library_id, docs);
                                            return Ok(Message::assistant(full_response));
                                        },
                                        Err(e) => {
                                            result.push_str(&format!("Failed to get documentation for '{}': {}\n", library_id, e));
                                        }
                                    }
                                }
                            },
                            _ => {
                                result.push_str(&format!("Unsupported tool call: {}\n", function_name));
                            }
                        }
                    }
                }
                
                if result.is_empty() && choice.message.content.is_some() {
                    choice.message.content.clone().unwrap_or_default()
                } else {
                    result
                }
            } else {
                choice.message.content.clone().unwrap_or_default()
            };
            
            Ok(Message::assistant(content))
        } else {
            Err(anyhow!("No choices in API response"))
        }
    }

    fn get_tools(&self) -> Vec<Value> {
        vec![
            json!({
                "type": "function",
                "function": {
                    "name": "mcp_context7_resolve_library_id",
                    "description": "Required first step: Resolves a general package name into a Context7-compatible library ID. Must be called before using 'get-library-docs' to retrieve a valid Context7-compatible library ID.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "libraryName": {
                                "type": "string",
                                "description": "Library name to search for and retrieve a Context7-compatible library ID."
                            }
                        },
                        "required": ["libraryName"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "mcp_context7_get_library_docs",
                    "description": "Fetches up-to-date documentation for a library. You must call 'resolve-library-id' first to obtain the exact Context7-compatible library ID required to use this tool.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "context7CompatibleLibraryID": {
                                "type": "string",
                                "description": "Exact Context7-compatible library ID (e.g., 'mongodb/docs', 'vercel/nextjs') retrieved from 'resolve-library-id'."
                            },
                            "tokens": {
                                "type": "number",
                                "description": "Maximum number of tokens of documentation to retrieve (default: 5000). Higher values provide more context but consume more tokens."
                            },
                            "topic": {
                                "type": "string",
                                "description": "Topic to focus documentation on (e.g., 'hooks', 'routing')."
                            }
                        },
                        "required": ["context7CompatibleLibraryID"]
                    }
                }
            })
        ]
    }
} 
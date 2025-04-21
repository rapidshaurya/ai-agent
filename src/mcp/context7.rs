use anyhow::{Result, anyhow};
use async_process::{Command, Child};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;
use std::time::Duration;
use tokio::time;
use tracing::{debug, error, info, warn};

use crate::config::Config;

static CHILD_PROCESS: OnceCell<Mutex<Option<Child>>> = OnceCell::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveLibraryIdRequest {
    pub library_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLibraryDocsRequest {
    pub context7_compatible_library_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
}

pub async fn ensure_mcp_server_running(config: &Config) -> Result<()> {
    if CHILD_PROCESS.get().is_none() {
        let mutex = Mutex::new(None);
        CHILD_PROCESS.set(mutex).map_err(|_| anyhow!("Failed to set CHILD_PROCESS"))?;
    }

    let mutex = CHILD_PROCESS.get().unwrap();
    let mut guard = mutex.lock().unwrap();

    if guard.is_none() {
        info!("Starting MCP server for Context7...");
        match Command::new(&config.mcp_servers.context7.command)
            .args(&config.mcp_servers.context7.args)
            .spawn() {
                Ok(child) => {
                    *guard = Some(child);
                    
                    // Allow time for the MCP server to start
                    drop(guard);
                    time::sleep(Duration::from_secs(2)).await;
                    info!("MCP server for Context7 started");
                },
                Err(e) => {
                    error!("Failed to start MCP server: {}", e);
                    warn!("Continuing without MCP server - some functionality may be limited");
                    return Ok(());
                }
            }
    }

    Ok(())
}

pub async fn stop_mcp_server() -> Result<()> {
    if let Some(mutex) = CHILD_PROCESS.get() {
        let mut guard = mutex.lock().unwrap();
        if let Some(mut child) = guard.take() {
            info!("Stopping MCP server for Context7...");
            if let Err(e) = child.kill() {
                error!("Failed to kill MCP server process: {}", e);
            }
            
            // Wait for process to exit
            match child.status().await {
                Ok(status) => {
                    info!("MCP server process exited with status: {}", status);
                },
                Err(e) => {
                    error!("Failed to get MCP server process status: {}", e);
                }
            }
        }
    }
    
    Ok(())
}

pub async fn resolve_library_id(library_name: String) -> Result<String> {
    let request = ResolveLibraryIdRequest { library_name };
    let response = call_context7_api("mcp_context7_resolve_library_id", request).await?;
    
    if let Some(id) = response.get("libraryId").and_then(|v| v.as_str()) {
        Ok(id.to_string())
    } else {
        Err(anyhow!("Failed to resolve library ID from response: {:?}", response))
    }
}

pub async fn get_library_docs(library_id: String, tokens: Option<u32>, topic: Option<String>) -> Result<String> {
    let request = GetLibraryDocsRequest {
        context7_compatible_library_id: library_id,
        tokens,
        topic,
    };
    
    let response = call_context7_api("mcp_context7_get_library_docs", request).await?;
    
    if let Some(docs) = response.get("documentation").and_then(|v| v.as_str()) {
        Ok(docs.to_string())
    } else {
        Err(anyhow!("Failed to get library documentation from response: {:?}", response))
    }
}

async fn call_context7_api<T: Serialize>(method: &str, params: T) -> Result<Value> {
    let client = reqwest::Client::new();
    
    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    
    debug!("Calling Context7 API: {} with params: {:?}", method, serde_json::to_string(&params)?);
    
    // Try to connect to the MCP server a few times, with a delay between attempts
    let max_retries = 3;
    let mut last_error = None;
    
    for attempt in 1..=max_retries {
        match client.post("http://localhost:3005/jsonrpc")
            .json(&request_body)
            .send()
            .await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        let response_json: Value = response.json().await?;
                        
                        if let Some(error) = response_json.get("error") {
                            error!("Context7 API error: {:?}", error);
                            return Err(anyhow!("Context7 API error: {:?}", error));
                        }
                        
                        if let Some(result) = response_json.get("result") {
                            return Ok(result.clone());
                        }
                        
                        return Err(anyhow!("Invalid Context7 API response: {:?}", response_json));
                    } else {
                        let error_text = response.text().await?;
                        last_error = Some(anyhow!("Context7 API HTTP error: {} - {}", status, error_text));
                    }
                },
                Err(e) => {
                    last_error = Some(anyhow!("Failed to connect to Context7 API: {}", e));
                }
            }
        
        if attempt < max_retries {
            warn!("Failed to call Context7 API, retrying in 1 second (attempt {}/{})", attempt, max_retries);
            time::sleep(Duration::from_secs(1)).await;
        }
    }
    
    Err(last_error.unwrap_or_else(|| anyhow!("Failed to call Context7 API after {} attempts", max_retries)))
} 
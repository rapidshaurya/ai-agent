use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use tracing::{error};
use colored::*;
use std::io::{self, Write};

use crate::agent::{Conversation, ConversationList, Message, OpenAIAgent, Role};
use crate::config::Config;
use crate::mcp;

const WELCOME_MESSAGE: &str = r#"
╭───────────────────────────────────────────╮
│                                           │
│   AI Agent with Context7 MCP Integration  │
│                                           │
╰───────────────────────────────────────────╯

Type your questions. Use these commands:
  !help   - Show this help message
  !exit   - Exit the chat
  !new    - Start a new conversation
  !list   - List saved conversations
  !load   - Load a conversation by ID
  !clear  - Clear the current conversation

"#;

const HELP_MESSAGE: &str = r#"Available commands:
  !help   - Show this help message
  !exit   - Exit the chat
  !new    - Start a new conversation
  !list   - List saved conversations
  !load   - Load a conversation by ID
  !clear  - Clear the current conversation
"#;

pub async fn start_chat() -> Result<()> {
    let config = Config::load()?;
    
    // Initialize the agent
    let agent = OpenAIAgent::new(config.clone());
    
    // Initialize the conversation list
    let list_path = config.history_path.join("conversations.json");
    let mut conversation_list = ConversationList::load_from_file(&list_path).unwrap_or_else(|_| ConversationList::new());
    
    // Initialize or load a conversation
    let mut current_conversation = Conversation::new("New Conversation".to_string());
    
    // Add a system message
    current_conversation.add_message(Message::system(
        "You are an AI assistant with access to Context7 libraries. You can help users \
        by providing documentation and assistance related to various programming libraries. \
        To use a library, you'll first need to resolve its ID and then fetch its documentation.".to_string()
    ));
    
    // Initialize readline
    let mut rl = DefaultEditor::new()?;
    
    // Display welcome message
    println!("{}", WELCOME_MESSAGE);
    
    // Try to start the MCP server, but don't fail if it can't start
    if let Err(e) = mcp::ensure_mcp_server_running(&config).await {
        println!("Note: Context7 MCP server could not be started: {}", e);
        println!("Some functionality may be limited. Continuing without Context7 integration.");
    }
    
    // Main REPL loop
    loop {
        match rl.readline("You: ") {
            Ok(line) => {
                let trimmed = line.trim();
                
                // Handle commands
                if trimmed.starts_with('!') {
                    match trimmed {
                        "!help" => {
                            println!("{}", HELP_MESSAGE);
                            continue;
                        },
                        "!exit" => {
                            println!("Goodbye!");
                            
                            // Save the current conversation
                            save_conversation(&mut current_conversation, &mut conversation_list, &config)?;
                            
                            // Try to stop the MCP server, but don't fail if it's not running
                            let _ = mcp::stop_mcp_server().await;
                            
                            break;
                        },
                        "!new" => {
                            // Save the current conversation
                            save_conversation(&mut current_conversation, &mut conversation_list, &config)?;
                            
                            // Create a new conversation
                            current_conversation = Conversation::new("New Conversation".to_string());
                            current_conversation.add_message(Message::system(
                                "You are an AI assistant with access to Context7 libraries. You can help users \
                                by providing documentation and assistance related to various programming libraries. \
                                To use a library, you'll first need to resolve its ID and then fetch its documentation.".to_string()
                            ));
                            
                            println!("Started a new conversation");
                            continue;
                        },
                        "!list" => {
                            list_conversations(&conversation_list);
                            continue;
                        },
                        "!load" => {
                            println!("Enter conversation ID to load:");
                            let id = rl.readline("ID: ")?;
                            
                            // Find the ID first, then clone it to avoid borrowing issues
                            let found_id = conversation_list.conversations.iter()
                                .find(|c| c.id == id)
                                .map(|summary| (summary.id.clone(), summary.title.clone()));
                            
                            if let Some((conversation_id, title)) = found_id {
                                let conv_path = config.history_path.join(format!("{}.json", conversation_id));
                                match Conversation::load_from_file(&conv_path) {
                                    Ok(conversation) => {
                                        // Save the current conversation first
                                        save_conversation(&mut current_conversation, &mut conversation_list, &config)?;
                                        
                                        // Load the selected conversation
                                        current_conversation = conversation;
                                        println!("Loaded conversation: {}", title);
                                    },
                                    Err(e) => {
                                        println!("Error loading conversation: {}", e);
                                    }
                                }
                            } else {
                                println!("Conversation not found with ID: {}", id);
                            }
                            continue;
                        },
                        "!clear" => {
                            // Create a new conversation with the same ID
                            let id = current_conversation.id.clone();
                            current_conversation = Conversation::new("New Conversation".to_string());
                            current_conversation.id = id;
                            current_conversation.add_message(Message::system(
                                "You are an AI assistant with access to Context7 libraries. You can help users \
                                by providing documentation and assistance related to various programming libraries. \
                                To use a library, you'll first need to resolve its ID and then fetch its documentation.".to_string()
                            ));
                            
                            println!("Conversation cleared");
                            continue;
                        },
                        _ => {
                            println!("Unknown command. Type !help for available commands.");
                            continue;
                        }
                    }
                }
                
                // Skip empty lines
                if trimmed.is_empty() {
                    continue;
                }
                
                // Add user message
                let user_message = Message::user(trimmed.to_string());
                current_conversation.add_message(user_message);
                
                // Show static thinking indicator instead of animation
                println!("{} {}", "AI:".yellow().bold(), "Thinking...");
                io::stdout().flush()?;
                
                // Get response from agent
                match agent.chat(&current_conversation).await {
                    Ok(response) => {
                        // Print the response (no need to clear previous line)
                        println!("{} {}", "AI:".green().bold(), response.content);
                        
                        // Add the response to the conversation
                        current_conversation.add_message(response);
                        
                        // Auto-save the conversation after each exchange
                        // Only save periodically (every 3 messages) to reduce disk I/O
                        if current_conversation.messages.len() % 3 == 0 {
                            let conv_path = config.history_path.join(format!("{}.json", current_conversation.id));
                            if let Err(e) = current_conversation.save_to_file(&conv_path) {
                                error!("Failed to save conversation: {}", e);
                            }
                            
                            // Update the conversation list
                            conversation_list.add_conversation(&current_conversation);
                            let list_path = config.history_path.join("conversations.json");
                            if let Err(e) = conversation_list.save_to_file(&list_path) {
                                error!("Failed to save conversation list: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        // Print the error (no need to clear previous line)
                        println!("{} Error: {}", "AI:".red().bold(), e);
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C pressed. Type !exit to quit.");
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D pressed, exiting...");
                
                // Save the current conversation
                save_conversation(&mut current_conversation, &mut conversation_list, &config)?;
                
                // Try to stop the MCP server, but don't fail if it's not running
                let _ = mcp::stop_mcp_server().await;
                
                break;
            },
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
    
    Ok(())
}

fn save_conversation(
    conversation: &mut Conversation,
    conversation_list: &mut ConversationList,
    config: &Config
) -> Result<()> {
    // Don't save empty conversations
    if conversation.messages.len() <= 1 {
        return Ok(());
    }
    
    // Set a better title based on the first user message
    if conversation.title == "New Conversation" {
        if let Some(first_user_msg) = conversation.messages.iter().find(|m| matches!(m.role, Role::User)) {
            let title = if first_user_msg.content.len() > 50 {
                format!("{}...", &first_user_msg.content[..47])
            } else {
                first_user_msg.content.clone()
            };
            conversation.title = title;
        }
    }
    
    // Save the conversation
    let conv_path = config.history_path.join(format!("{}.json", conversation.id));
    conversation.save_to_file(&conv_path)?;
    
    // Update the conversation list
    conversation_list.add_conversation(&conversation);
    let list_path = config.history_path.join("conversations.json");
    conversation_list.save_to_file(&list_path)?;
    
    Ok(())
}

fn list_conversations(conversation_list: &ConversationList) {
    if conversation_list.conversations.is_empty() {
        println!("No saved conversations");
        return;
    }
    
    println!("{}", "Saved Conversations:".bold());
    println!("{}", "─".repeat(80));
    println!("{:<36} │ {:<30} │ {:<10}", "ID", "Title", "Messages");
    println!("{}", "─".repeat(80));
    
    for (i, summary) in conversation_list.conversations.iter().enumerate() {
        println!("{:<36} │ {:<30} │ {:<10}",
            summary.id,
            if summary.title.len() > 28 { format!("{}...", &summary.title[..25]) } else { summary.title.clone() },
            summary.message_count
        );
        
        if i < conversation_list.conversations.len() - 1 {
            println!("{}", "─".repeat(80));
        }
    }
} 
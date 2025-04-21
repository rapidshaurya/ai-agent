mod agent;
mod cli;
mod config;
mod mcp;

use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, prelude::*};
use tracing_subscriber::EnvFilter;
use anyhow::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a chat session with the AI
    Chat,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Handle commands
    match cli.command {
        Some(Commands::Chat) => {
            cli::start_chat().await?;
        }
        None => {
            // Default to chat if no command is provided
            cli::start_chat().await?;
        }
    }
    
    Ok(())
}

# AI Agent with Context7 MCP Integration

This Rust-based AI agent can connect to the Context7 MCP server to provide documentation and assistance for programming libraries.

## Features

- Uses OpenAI API for chat capabilities
- Integrates with Context7 MCP server for up-to-date library documentation
- CLI interface with conversation history management
- Environment variable configuration

## Prerequisites

- Rust (latest stable version)
- Node.js (for running the Context7 MCP server)
- OpenAI API key

## Setup

1. Clone this repository
   ```
   git clone <repository-url>
   cd ai-agent
   ```

2. Copy the example environment file and fill in your credentials:
   ```
   cp .env.example .env
   ```

3. Edit the `.env` file with your OpenAI API key:
   ```
   OPENAI_API_KEY=your_openai_api_key_here
   OPENAI_API_BASE_URL=https://api.openai.com/v1
   OPENAI_API_MODEL=gpt-4-turbo
   ```

4. Build the project:
   ```
   cargo build --release
   ```

## Usage

Run the agent in chat mode:

```
cargo run --release
```

Or use the built binary directly:

```
./target/release/ai-agent
```

### CLI Commands

Inside the chat interface, you can use the following commands:

- `!help` - Show help message
- `!exit` - Exit the chat
- `!new` - Start a new conversation
- `!list` - List saved conversations
- `!load` - Load a conversation by ID
- `!clear` - Clear the current conversation

## How it Works

1. The agent starts the Context7 MCP server in the background
2. When you ask a question about a library, the agent can:
   - Resolve the library ID using Context7
   - Fetch up-to-date documentation for the library
   - Provide answers based on the documentation
3. Conversations are saved automatically in `~/.ai-agent/history/`

## Configuration

All configuration is done through environment variables:

- `OPENAI_API_KEY`: Your OpenAI API key
- `OPENAI_API_BASE_URL`: Base URL for OpenAI API (default: https://api.openai.com/v1)
- `OPENAI_API_MODEL`: OpenAI model to use (default: gpt-4-turbo)
- `AGENT_NAME`: Name of the agent (default: ai-assistant)
- `HISTORY_PATH`: Path to store conversation history (default: ~/.ai-agent/history)

## License

MIT 
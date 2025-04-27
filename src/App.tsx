import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import { v4 as uuidv4 } from 'uuid';
import ChatMessage from './components/ChatMessage';
// import UserInput from './components/UserInput';
// import ChatContainer from './components/ChatContainer';

interface Message {
  role: string;
  content: string;
}

interface Conversation {
  id: string;
  title: string;
  messages: Message[];
  createdAt: string;
}

interface ChatSettings {
  openai_api_key: string;
  openai_api_base_url: string;
  openai_api_model: string;
  history_path: string;
}

function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState<string>("");
  const [isInitialized, setIsInitialized] = useState<boolean>(false);
  const [showSettings, setShowSettings] = useState<boolean>(false);
  const [showHistory, setShowHistory] = useState<boolean>(false);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [currentConversationId, setCurrentConversationId] = useState<string>("");
  const [settings, setSettings] = useState<ChatSettings>({
    openai_api_key: "",
    openai_api_base_url: "https://api.groq.com/openai/v1",
    openai_api_model: "llama3-70b-8192",
    history_path: "~/.ai-agent/history"
  });
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Load settings and conversations from localStorage on initial render
  useEffect(() => {
    const savedSettings = localStorage.getItem("aiAgentSettings");
    if (savedSettings) {
      const parsedSettings = JSON.parse(savedSettings);
      setSettings(parsedSettings);
      
      // Auto-initialize if we have stored settings
      if (parsedSettings.openai_api_key) {
        initializeAgent(parsedSettings);
      } else {
        setShowSettings(true);
      }
    } else {
      // No saved settings, show settings dialog
      loadDefaultSettings();
      setShowSettings(true);
    }

    // Load saved conversations
    const savedConversations = localStorage.getItem("aiAgentConversations");
    if (savedConversations) {
      const parsedConversations = JSON.parse(savedConversations);
      setConversations(parsedConversations);
    }
  }, []);

  // Save conversations to localStorage whenever they change
  useEffect(() => {
    if (conversations.length > 0) {
      localStorage.setItem("aiAgentConversations", JSON.stringify(conversations));
    }
  }, [conversations]);

  // Scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);
  
  // Add ESC key listener to close settings modal
  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && showSettings) {
        setShowSettings(false);
      }
    };
    
    window.addEventListener('keydown', handleEsc);
    
    return () => {
      window.removeEventListener('keydown', handleEsc);
    };
  }, [showSettings]);
  
  // Initialize agent with settings
  const initializeAgent = async (settingsToUse: ChatSettings) => {
    try {
      const result = await invoke<boolean>("initialize_agent", { settings: settingsToUse });
      setIsInitialized(result);
      
      if (result) {
        // Save settings to localStorage
        localStorage.setItem("aiAgentSettings", JSON.stringify(settingsToUse));
        
        // Create a new conversation if none exists
        if (!currentConversationId) {
          createNewConversation();
        }
        
        // Load conversation history
        const history = await invoke<Message[]>("get_conversation_history");
        if (history && history.length > 0) {
          setMessages(history);
        }
      }
    } catch (error) {
      console.error("Failed to initialize agent:", error);
      alert(`Failed to initialize agent: ${error}`);
    }
  };
  
  // Load default settings
  const loadDefaultSettings = async () => {
    try {
      const defaultSettings = await invoke<ChatSettings>("get_default_settings");
      setSettings(defaultSettings);
    } catch (error) {
      console.error("Failed to load default settings:", error);
    }
  };

  // Create a new conversation
  const createNewConversation = () => {
    const newId = uuidv4();
    const newConversation: Conversation = {
      id: newId,
      title: `New Chat (${new Date().toLocaleString()})`,
      messages: [],
      createdAt: new Date().toISOString(),
    };
    
    setConversations(prev => {
      const updated = [...prev, newConversation];
      localStorage.setItem("aiAgentConversations", JSON.stringify(updated));
      return updated;
    });
    
    setCurrentConversationId(newId);
    setMessages([]);
  };

  // Load a conversation from history
  const loadConversation = (id: string) => {
    const conversation = conversations.find(conv => conv.id === id);
    if (conversation) {
      setMessages(conversation.messages);
      setCurrentConversationId(id);
      setShowHistory(false);
    }
  };

  // Delete a conversation from history
  const deleteConversation = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const updatedConversations = conversations.filter(conv => conv.id !== id);
    setConversations(updatedConversations);
    localStorage.setItem("aiAgentConversations", JSON.stringify(updatedConversations));
    
    if (currentConversationId === id) {
      if (updatedConversations.length > 0) {
        loadConversation(updatedConversations[0].id);
      } else {
        createNewConversation();
      }
    }
  };

  // Send a message to the AI
  const sendMessage = async () => {
    if (!input.trim() || !isInitialized) return;
    
    // Add user message to the chat
    const userMessage: Message = { role: "user", content: input };
    setMessages(prev => [...prev, userMessage]);
    setInput("");
    
    try {
      // Send message to backend
      const updatedConversation = await invoke<Message[]>("send_message", { message: input });
      setMessages(updatedConversation);
      
      // Update conversation in local storage
      setConversations(prev => {
        const updated = prev.map(conv => {
          if (conv.id === currentConversationId) {
            // Set title to first few words of the first user message
            let title = conv.title;
            if (updatedConversation.length > 0) {
              const firstUserMessage = updatedConversation.find(msg => msg.role === "user");
              if (firstUserMessage && conv.title.startsWith("New Chat")) {
                const words = firstUserMessage.content.split(" ").slice(0, 5).join(" ");
                title = words + (words.length < firstUserMessage.content.split(" ").length ? "..." : "");
              }
            }
            return { ...conv, messages: updatedConversation, title };
          }
          return conv;
        });
        localStorage.setItem("aiAgentConversations", JSON.stringify(updated));
        return updated;
      });
    } catch (error) {
      console.error("Failed to send message:", error);
      // Add error message to chat
      setMessages(prev => [...prev, { role: "assistant", content: `Error: ${error}` }]);
    }
  };
  
  // Save settings and initialize agent
  const saveSettings = () => {
    initializeAgent(settings);
    setShowSettings(false);
  };

  return (
    <div className="App">
      <div className={`history-sidebar ${showHistory ? 'show' : ''}`}>
        <div className="history-header">
          <h2>Conversation History</h2>
          <button className="close-history" onClick={() => setShowHistory(false)}>×</button>
        </div>
        <button className="new-chat-button" onClick={createNewConversation}>
          + New Chat
        </button>
        <div className="history-list">
          {conversations.length === 0 ? (
            <div className="no-history">No conversations yet</div>
          ) : (
            conversations.map((convo) => (
              <div 
                key={convo.id} 
                className={`history-item ${convo.id === currentConversationId ? 'active' : ''}`}
                onClick={() => loadConversation(convo.id)}
              >
                <div className="history-title">{convo.title}</div>
                <button 
                  className="delete-conversation"
                  onClick={(e) => {
                    e.stopPropagation();
                    deleteConversation(convo.id, e);
                  }}
                >
                  ×
                </button>
              </div>
            ))
          )}
        </div>
      </div>

      <div className="app-header">
        <div className="left-controls">
          <button className="history-button" onClick={() => setShowHistory(true)}>
            History
          </button>
          <button className="new-chat-button-small" onClick={createNewConversation}>
            New Chat
          </button>
        </div>
        <h1>AI Chat Assistant</h1>
        <div className="right-controls">
          <button className="settings-button" onClick={() => setShowSettings(!showSettings)}>
            ⚙️
          </button>
        </div>
      </div>

      {showSettings && (
        <div className="settings-modal">
          <div className="settings-content">
            <div className="settings-header">
              <h2>AI Agent Settings</h2>
              <button className="close-settings" onClick={() => setShowSettings(false)}>×</button>
            </div>
            
            <div className="settings-field">
              <label htmlFor="api-key">API Key:</label>
              <input
                id="api-key"
                type="password"
                value={settings.openai_api_key}
                onChange={(e) => setSettings({...settings, openai_api_key: e.target.value})}
                placeholder="Enter your OpenAI API key"
              />
            </div>
            
            <div className="settings-field">
              <label htmlFor="api-url">API Base URL:</label>
              <input
                id="api-url"
                type="text"
                value={settings.openai_api_base_url}
                onChange={(e) => setSettings({...settings, openai_api_base_url: e.target.value})}
                placeholder="API base URL"
              />
            </div>
            
            <div className="settings-field">
              <label htmlFor="api-model">Model:</label>
              <input
                id="api-model"
                type="text"
                value={settings.openai_api_model}
                onChange={(e) => setSettings({...settings, openai_api_model: e.target.value})}
                placeholder="Model name"
              />
            </div>
            
            <div className="settings-field">
              <label htmlFor="history-path">History Path:</label>
              <div className="input-with-button">
                <input
                  id="history-path"
                  type="text"
                  value={settings.history_path}
                  onChange={(e) => setSettings({...settings, history_path: e.target.value})}
                  placeholder="Path to save conversation history"
                />
                <button 
                  className="browse-button" 
                  onClick={async () => {
                    try {
                      // Use Tauri dialog API to select a directory
                      const selected = await invoke<string>("select_folder", {
                        title: "Select History Folder"
                      });
                      if (selected) {
                        setSettings({...settings, history_path: selected});
                      }
                    } catch (error) {
                      console.error("Failed to select folder:", error);
                    }
                  }}
                >
                  Browse
                </button>
              </div>
            </div>
            
            <div className="settings-buttons">
              <span className="settings-hint">Press ESC to close</span>
              <div className="button-group">
                <button className="cancel-button" onClick={() => setShowSettings(false)}>Cancel</button>
                <button className="save-button" onClick={saveSettings}>Save & Connect</button>
              </div>
            </div>
          </div>
        </div>
      )}

      <main className="chat-container">
        {messages.length === 0 ? (
          <div className="empty-chat">
            <h2>Start a new conversation</h2>
            <p>Type a message below to begin chatting with the AI assistant</p>
          </div>
        ) : (
          <div className="messages-container">
            {messages.map((msg, index) => (
              <ChatMessage key={index} message={msg} />
            ))}
            <div ref={messagesEndRef} />
          </div>
        )}
        
        <div className="input-container">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                sendMessage();
              }
            }}
            placeholder={isInitialized ? "Type your message..." : "Please configure settings first..."}
            disabled={!isInitialized}
          />
          <button 
            onClick={sendMessage}
            disabled={!isInitialized || !input.trim()}
          >
            Send
          </button>
        </div>
      </main>
    </div>
  );
}

export default App;

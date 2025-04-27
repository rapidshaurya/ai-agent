import React, { useState, useRef, useEffect } from 'react';
import ChatMessage from './ChatMessage';
import UserInput from './UserInput';

interface Message {
  role: 'user' | 'assistant';
  content: string;
}

interface ChatContainerProps {
  initialMessages?: Message[];
  onSendMessage?: (message: string) => Promise<void>;
  isLoading?: boolean;
}

const ChatContainer: React.FC<ChatContainerProps> = ({
  initialMessages = [],
  onSendMessage,
  isLoading = false
}) => {
  const [messages, setMessages] = useState<Message[]>(initialMessages);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const handleSendMessage = async (content: string) => {
    const userMessage: Message = { role: 'user', content };
    setMessages(prev => [...prev, userMessage]);
    
    if (onSendMessage) {
      await onSendMessage(content);
    }
  };

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  return (
    <div className="chat-container">
      <div className="messages-container">
        {messages.map((message, index) => (
          <ChatMessage 
            key={index} 
            message={message}
          />
        ))}
        {isLoading && (
          <div className="loading-indicator">
            <span>AI is thinking...</span>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>
      <UserInput 
        onSendMessage={handleSendMessage}
        disabled={isLoading}
      />
    </div>
  );
};

export default ChatContainer; 
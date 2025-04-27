import React, { useState, KeyboardEvent, ChangeEvent } from 'react';

interface UserInputProps {
  onSendMessage: (message: string) => void;
  placeholder?: string;
  disabled?: boolean;
}

const UserInput: React.FC<UserInputProps> = ({ 
  onSendMessage, 
  placeholder = "Type your message here...", 
  disabled = false 
}) => {
  const [input, setInput] = useState('');

  const handleInputChange = (e: ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  const sendMessage = () => {
    if (input.trim() && !disabled) {
      onSendMessage(input);
      setInput('');
    }
  };

  return (
    <div className="user-input-container">
      <textarea
        className="user-input"
        value={input}
        onChange={handleInputChange}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={disabled}
        rows={1}
      />
      <button 
        className="send-button"
        onClick={sendMessage}
        disabled={!input.trim() || disabled}
      >
        Send
      </button>
    </div>
  );
};

export default UserInput; 
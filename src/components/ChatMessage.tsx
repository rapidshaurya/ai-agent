import React, { useEffect, useRef } from 'react';
import hljs from 'highlight.js';
import 'highlight.js/styles/github-dark.css'; // Import a dark theme

interface ChatMessageProps {
  message: {
    role: string;
    content: string;
  };
}

const ChatMessage: React.FC<ChatMessageProps> = ({ message }) => {
  const codeRefs = useRef<(HTMLElement | null)[]>([]);
  
  // Apply syntax highlighting after component mounts or updates
  useEffect(() => {
    codeRefs.current.forEach(el => {
      if (el) {
        hljs.highlightElement(el);
      }
    });
  }, [message.content]);

  // Function to process content and detect code blocks
  const processContent = (content: string) => {
    // Split by code block markers (```language...```)
    const parts = content.split(/(```[\w-]*\n[\s\S]*?\n```)/g);
    let codeBlockCount = 0;
    
    return parts.map((part, index) => {
      // Check if this part is a code block
      const codeMatch = part.match(/```([\w-]*)\n([\s\S]*?)\n```/);
      
      if (codeMatch) {
        const [, language, code] = codeMatch;
        const languageClass = language || 'plaintext';
        
        return (
          <pre key={index} className="code-block">
            {language && <div className="code-language">{language}</div>}
            <code 
              ref={el => codeRefs.current[codeBlockCount++] = el}
              className={`language-${languageClass}`}
            >
              {code}
            </code>
          </pre>
        );
      }
      
      // For regular text, split by newlines and render paragraphs
      if (part.trim()) {
        return part.split('\n').map((line, i) => (
          <p key={`${index}-${i}`}>{line}</p>
        ));
      }
      
      return null;
    });
  };

  return (
    <div className={`message ${message.role === 'user' ? 'user-message' : 'assistant-message'}`}>
      <div className="message-content">
        {processContent(message.content)}
      </div>
    </div>
  );
};

export default ChatMessage; 
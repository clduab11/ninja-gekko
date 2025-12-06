import { useMemo } from 'react';
import clsx from 'clsx';
import { ChatMessage } from '../../types';

interface Props {
  messages: ChatMessage[];
}

const roleStyles: Record<ChatMessage['role'], string> = {
  user: 'bg-white/5 border border-white/10',
  assistant: 'bg-accentSoft/10 border border-accentSoft/30',
  system: 'bg-panel border border-border/60'
};

function formatTimestamp(value: string) {
  return new Date(value).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

const ChatConversation = ({ messages }: Props) => {
  const rendered = useMemo(() => messages.slice(-80), [messages]);
  return (
    <div 
      className="flex-1 overflow-y-auto px-6 py-6 space-y-4"
      data-testid="chat-conversation"
      role="log"
      aria-label="Chat message history with Gordon"
      aria-live="polite"
      aria-relevant="additions"
    >
      {rendered.map((message, index) => (
        <article 
          key={message.id} 
          className={clsx('rounded-xl p-4 shadow-inner', roleStyles[message.role])}
          data-testid={`chat-message-${message.role}`}
          aria-label={`${message.role === 'assistant' ? 'Gordon' : message.role} message`}
        >
          <header className="flex items-center justify-between text-xs uppercase tracking-[0.3em] text-white/40">
            <span data-testid={`message-role-${index}`}>{message.role === 'assistant' ? 'Gordon' : message.role}</span>
            <time 
              dateTime={message.timestamp}
              data-testid={`message-timestamp-${index}`}
            >
              {formatTimestamp(message.timestamp)}
            </time>
          </header>
          <p 
            className="mt-3 text-sm leading-relaxed text-white/90 whitespace-pre-line"
            data-testid={`message-content-${index}`}
          >
            {message.content}
          </p>
          {message.citations?.length ? (
            <ul 
              className="mt-3 flex flex-wrap gap-2 text-[11px] text-white/60"
              data-testid={`message-citations-${index}`}
              role="list"
              aria-label="Message citations"
            >
              {message.citations.map((citation, idx) => (
                <li 
                  key={idx} 
                  className="rounded-full border border-accent/40 px-3 py-1"
                  data-testid={`citation-${index}-${idx}`}
                >
                  {citation.type === 'external' ? (
                    <a 
                      href={citation.url} 
                      target="_blank" 
                      rel="noreferrer" 
                      className="hover:text-accent"
                      aria-label={`External citation: ${citation.title}`}
                    >
                      {citation.title}
                    </a>
                  ) : (
                    <span aria-label={`Inline citation from ${citation.source}`}>
                      {citation.source}: {citation.detail}
                    </span>
                  )}
                </li>
              ))}
            </ul>
          ) : null}
        </article>
      ))}
    </div>
  );
};

export default ChatConversation;

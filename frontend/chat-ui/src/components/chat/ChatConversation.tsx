import { useMemo } from 'react';
import clsx from 'clsx';
import { Bot, User, Terminal } from 'lucide-react';
import { ChatMessage } from '../../types';

interface Props {
  messages: ChatMessage[];
}

// Bloomberg / Terminal inspired styles
const roleStyles: Record<ChatMessage['role'], string> = {
  user: 'bg-slate-800/80 border-slate-700/50 text-slate-200 ml-12',
  assistant: 'bg-slate-900/90 border-emerald-900/30 text-emerald-50 shadow-[0_0_20px_rgba(16,185,129,0.05)] mr-12 border-l-2 border-l-emerald-500',
  system: 'bg-red-950/20 border-red-900/30 text-red-200 mx-auto max-w-xl text-center'
};

function formatTimestamp(value: string) {
  return new Date(value).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

const ChatConversation = ({ messages }: Props) => {
  const rendered = useMemo(() => messages.slice(-80), [messages]);
  return (
    <div 
      className="flex-1 overflow-y-auto px-4 py-4 space-y-6 font-mono text-sm"
      data-testid="chat-message-list"
      role="log"
    >
      {rendered.map((message, index) => {
        const isGordon = message.role === 'assistant';
        return (
          <article 
            key={message.id} 
            className={clsx('relative rounded p-4 border transition-all', roleStyles[message.role])}
            data-testid={`msg-${message.role}-${index}`}
          >
            {/* Header */}
            <header className="mb-2 flex items-center gap-2 text-[10px] uppercase tracking-widest opacity-60">
                {isGordon ? (
                    <Bot className="h-3 w-3 text-emerald-500" />
                ) : (
                    <User className="h-3 w-3 text-slate-400" />
                )}
                <span className={isGordon ? "text-emerald-400 font-bold" : "text-slate-400"}>
                    {isGordon ? "GORDON.GEKKO_AI" : "TRADER_1"}
                </span>
                <span className="text-slate-600">|</span>
                <time className="text-slate-500">{formatTimestamp(message.timestamp)}</time>
            </header>

            {/* Content */}
            <div className={clsx("whitespace-pre-wrap leading-relaxed", isGordon ? "text-emerald-50/90" : "text-slate-200")}>
                {message.content}
            </div>

            {/* Citations / Footnotes */}
            {message.citations?.length ? (
              <div className="mt-4 border-t border-white/5 pt-2">
                 <div className="mb-1 flex items-center gap-1 text-[9px] uppercase tracking-widest text-slate-500">
                    <Terminal className="h-3 w-3" /> Data Sources
                 </div>
                 <div className="flex flex-wrap gap-2">
                    {message.citations.map((cit, idx) => (
                        <span key={idx} className="inline-flex items-center rounded border border-white/10 bg-black/20 px-2 py-0.5 text-[10px] text-slate-400">
                            {cit.source}
                        </span>
                    ))}
                 </div>
              </div>
            ) : null}
          </article>
        );
      })}
    </div>
  );
};

export default ChatConversation;

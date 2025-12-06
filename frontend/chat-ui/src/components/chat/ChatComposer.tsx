import { FormEvent, useState } from 'react';
import { Loader2, Send, Upload } from 'lucide-react';

interface Props {
  disabled?: boolean;
  onSend: (prompt: string) => void;
}

const ChatComposer = ({ disabled, onSend }: Props) => {
  const [value, setValue] = useState('');

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!value.trim()) return;
    onSend(value.trim());
    setValue('');
  };

  return (
    <form 
      onSubmit={handleSubmit} 
      className="border-t border-border/60 p-4"
      data-testid="chat-composer"
      role="form"
      aria-label="Send message to Gordon"
    >
      <div className="rounded-xl border border-border/60 bg-panel px-4 py-3">
        <textarea
          className="h-24 w-full resize-none border-none bg-transparent text-sm text-white/90 outline-none"
          placeholder="Ask Gordon to orchestrate trades, research, or automation..."
          value={value}
          onChange={(event) => setValue(event.target.value)}
          disabled={disabled}
          data-testid="chat-input-field"
          aria-label="Type your message to Gordon"
          aria-disabled={disabled}
        />
        <div className="mt-3 flex items-center justify-between text-xs text-white/60">
          <label 
            className="flex cursor-pointer items-center gap-2 rounded-full border border-border/80 px-3 py-2 hover:border-accent/70"
            data-testid="btn-attach-file"
            aria-label="Attach files (CSV, PDF, MD)"
          >
            <Upload className="h-4 w-4" aria-hidden="true" />
            Attach (CSV, PDF, MD)
            <input 
              type="file" 
              className="hidden" 
              multiple 
              data-testid="input-file-upload"
              aria-label="Select files to attach"
            />
          </label>
          <button
            type="submit"
            className="flex items-center gap-2 rounded-full bg-accent px-4 py-2 font-semibold text-black disabled:cursor-not-allowed disabled:bg-white/40"
            disabled={disabled}
            data-testid="btn-send-message"
            role="button"
            aria-label={disabled ? 'Sending message...' : 'Send message'}
            aria-disabled={disabled}
          >
            {disabled ? (
              <Loader2 className="h-4 w-4 animate-spin" aria-hidden="true" />
            ) : (
              <Send className="h-4 w-4" aria-hidden="true" />
            )} Send
          </button>
        </div>
      </div>
    </form>
  );
};

export default ChatComposer;

import { FormEvent, useState } from 'react';
import { Loader2, Send, Upload, Wrench, BarChart2, Globe } from 'lucide-react';
import clsx from 'clsx';

interface Props {
  disabled?: boolean;
  onSend: (prompt: string) => void;
}

const ChatComposer = ({ disabled, onSend }: Props) => {
  const [value, setValue] = useState('');
  const [showTools, setShowTools] = useState(false);

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!value.trim()) return;
    onSend(value.trim());
    setValue('');
    setShowTools(false);
  };

  const insertToolCommand = (cmd: string) => {
      setValue(prev => `${prev} /${cmd} `);
      setShowTools(false);
  };

  return (
    <form 
      onSubmit={handleSubmit} 
      className="border-t border-white/5 bg-slate-900 p-2"
      data-testid="chat-composer"
    >
      <div className="relative rounded-lg border border-white/10 bg-slate-950 focus-within:border-emerald-500/50 focus-within:ring-1 focus-within:ring-emerald-500/20 transition-all">
        
        {/* Tools Menu Overlay */}
        {showTools && (
            <div className="absolute bottom-full left-0 mb-2 w-full rounded-lg border border-white/10 bg-slate-900 p-2 shadow-xl animate-in fade-in slide-in-from-bottom-2">
                <div className="mb-2 text-[10px] uppercase tracking-wider text-slate-500 font-bold px-1">Institutional Tools</div>
                <div className="grid grid-cols-3 gap-2">
                    <button type="button" onClick={() => insertToolCommand('oanda')} className="flex flex-col items-center gap-1 rounded bg-white/5 p-2 hover:bg-emerald-500/10 hover:text-emerald-400 transition-colors">
                        <Globe className="h-4 w-4" />
                        <span className="text-[10px]">OANDA Qry</span>
                    </button>
                    <button type="button" onClick={() => insertToolCommand('sentiment')} className="flex flex-col items-center gap-1 rounded bg-white/5 p-2 hover:bg-purple-500/10 hover:text-purple-400 transition-colors">
                        <BarChart2 className="h-4 w-4" />
                        <span className="text-[10px]">Sentiment</span>
                    </button>
                    <button type="button" onClick={() => insertToolCommand('swarm')} className="flex flex-col items-center gap-1 rounded bg-white/5 p-2 hover:bg-amber-500/10 hover:text-amber-400 transition-colors">
                        <Loader2 className="h-4 w-4" />
                        <span className="text-[10px]">Exec Swarm</span>
                    </button>
                </div>
            </div>
        )}

        <textarea
          className="h-20 w-full resize-none border-none bg-transparent p-3 text-sm font-mono text-slate-200 placeholder-slate-600 outline-none"
          placeholder="Direct line to Gordon..."
          value={value}
          onChange={(event) => setValue(event.target.value)}
          disabled={disabled}
        />
        
        <div className="flex items-center justify-between border-t border-white/5 bg-white/[0.02] px-2 py-1.5">
          <div className="flex items-center gap-1">
             <button
                type="button"
                onClick={() => setShowTools(!showTools)}
                className={`flex items-center gap-1.5 rounded px-2 py-1 text-[10px] font-bold uppercase tracking-wide transition-colors ${showTools ? 'bg-emerald-500/20 text-emerald-400' : 'text-slate-500 hover:text-slate-300'}`}
             >
                <Wrench className="h-3 w-3" /> Tools
             </button>
             <div className="h-4 w-px bg-white/10 mx-1" />
             <label className="flex cursor-pointer items-center gap-1.5 rounded px-2 py-1 text-[10px] font-bold uppercase tracking-wide text-slate-500 hover:text-slate-300 transition-colors">
                <Upload className="h-3 w-3" /> Attach
                <input type="file" className="hidden" multiple />
             </label>
          </div>

          <button
            type="submit"
            disabled={disabled || !value.trim()}
            className={clsx(
              "flex items-center gap-2 rounded px-3 py-1 text-[10px] font-bold uppercase tracking-widest transition-all",
              disabled || !value.trim() 
                ? "text-slate-600 cursor-not-allowed" 
                : "bg-emerald-600 text-white hover:bg-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.4)]"
            )}
          >
            {disabled ? <Loader2 className="h-3 w-3 animate-spin" /> : <Send className="h-3 w-3" />}
            Execute
          </button>
        </div>
      </div>
    </form>
  );
};

export default ChatComposer;

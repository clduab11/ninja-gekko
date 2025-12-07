import { BrainCircuit, Hand, ShieldAlert, Terminal, Activity, Zap, Search, Globe } from 'lucide-react';
import { pauseTrading, requestResearch, summonSwarm } from '../../services/api';

const ActionDashboard = () => {
  const handlePause = async () => {
    await pauseTrading({ duration_hours: 6 });
  };

  const handleSwarm = async () => {
    await summonSwarm({ task: 'sector rotation diagnostics' });
  };

  const handleResearch = async () => {
    await requestResearch({ query: 'Top volatility catalysts within 24h' });
  };

  // We are removing the Treasury widget from here as it should be in MarketRadar or Header to save space for ACTIONS.
  // This panel is now purely the "Control Center".

  return (
    <div className="flex h-full flex-col rounded-lg border border-white/5 bg-slate-900/40 p-1" data-testid="action-dashboard">
      <div className="mb-2 flex items-center px-4 py-2 border-b border-white/5">
        <Terminal className="mr-2 h-4 w-4 text-emerald-500" />
        <h3 className="text-xs font-bold uppercase tracking-[0.2em] text-white/60">Command Deck</h3>
      </div>
      
      <div className="grid flex-1 grid-cols-2 gap-2 p-2 overflow-y-auto">
          <CommandButton 
            onClick={handleSwarm}
            icon={<BrainCircuit className="h-5 w-5 text-purple-400" />}
            label="Swarm Logic"
            description="Parallel agent analysis"
            testId="btn-summon-swarm"
          />
          <CommandButton 
            onClick={handleResearch}
            icon={<Search className="h-5 w-5 text-blue-400" />}
            label="Deep Res"
            description="Sonar / Perplexity Scan"
            testId="btn-research-pulse"
          />
          <CommandButton 
            onClick={() => {}} 
            icon={<Globe className="h-5 w-5 text-amber-400" />}
            label="Macro View"
            description="Global liquidity map"
            testId="btn-macro-view"
            disabled
          />
           <CommandButton 
            onClick={() => {}} 
            icon={<Zap className="h-5 w-5 text-emerald-400" />}
            label="HFT Mode"
            description="Low-latency execution"
            testId="btn-hft-mode"
            disabled
          />
          <CommandButton 
            onClick={handlePause}
            icon={<Hand className="h-5 w-5 text-red-500" />}
            label="Kill Switch"
            description="Emergency halt (6h)"
            testId="btn-pause-trading"
            variant="danger"
            className="col-span-2"
          />
      </div>
    </div>
  );
};

interface CommandButtonProps {
    onClick: () => void;
    icon: JSX.Element;
    label: string;
    description: string;
    testId: string;
    variant?: 'default' | 'danger';
    disabled?: boolean;
    className?: string; // Allow col-span overriding
}

const CommandButton = ({ onClick, icon, label, description, testId, variant = 'default', disabled, className = '' }: CommandButtonProps) => (
    <button
        onClick={onClick}
        disabled={disabled}
        className={`
            group relative flex flex-col justify-center rounded-md border px-4 py-3 text-left transition-all
            ${disabled ? 'opacity-40 cursor-not-allowed border-white/5 bg-white/5' : 
              variant === 'danger' 
                ? 'border-red-900/30 bg-red-950/20 hover:bg-red-900/40 hover:border-red-500/50' 
                : 'border-white/10 bg-white/5 hover:bg-white/10 hover:border-emerald-500/30 hover:shadow-[0_0_15px_rgba(16,185,129,0.1)]'
            }
            ${className}
        `}
        data-testid={testId}
    >
        <div className="flex items-center justify-between mb-2">
            {icon}
            {!disabled && variant !== 'danger' && <div className="h-1.5 w-1.5 rounded-full bg-emerald-500 opacity-0 group-hover:opacity-100 shadow-[0_0_5px_#10b981] transition-opacity" />}
        </div>
        <div>
            <div className="font-bold text-sm text-slate-200 group-hover:text-white">{label}</div>
            <div className="text-[10px] text-slate-500 group-hover:text-slate-400">{description}</div>
        </div>
    </button>
);

export default ActionDashboard;

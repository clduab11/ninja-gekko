import { useState, useEffect } from 'react';
import { 
  BrainCircuit, 
  Hand, 
  ShieldAlert, 
  Terminal, 
  Activity, 
  Zap, 
  Search, 
  Globe,
  Play,
  Power
} from 'lucide-react';
import { 
  pauseTrading, 
  requestResearch, 
  summonSwarm, 
  engage, 
  windDown, 
  emergencyHalt, 
  setRiskThrottle, 
  getOrchestratorState 
} from '../../services/api';
import { OrchestratorState } from '../../types';

const ActionDashboard = () => {
  const [orchestratorState, setOrchestratorState] = useState<OrchestratorState | null>(null);
  const [riskValue, setRiskValue] = useState(100);
  const [showHaltingConfirm, setShowHaltingConfirm] = useState(false);

  const [showRecoveryConfirm, setShowRecoveryConfirm] = useState(false);

  useEffect(() => {
    // Initial fetch
    fetchState();
    // Poll every 5 seconds for status updates
    const interval = setInterval(fetchState, 5000);
    return () => clearInterval(interval);
  }, []);

  const fetchState = async () => {
    try {
      const state = await getOrchestratorState();
      setOrchestratorState(state);
      setRiskValue(state.risk_throttle * 100);
    } catch (e) {
      console.error("Failed to fetch orchestrator state", e);
    }
  };

  const handleEngageClick = () => {
    if (orchestratorState?.emergency_halt_active) {
      setShowRecoveryConfirm(true);
    } else {
      handleEngage();
    }
  };

  const handleEngage = async () => {
    try {
      const newState = await engage();
      setOrchestratorState(newState);
      setShowRecoveryConfirm(false);
    } catch (e) {
      console.error("Failed to engage", e);
    }
  };

  const handleWindDown = async () => {
    try {
      const newState = await windDown(60 * 60); // 1 hour default
      setOrchestratorState(newState);
    } catch (e) {
      console.error("Failed to wind down", e);
    }
  };

  const handleEmergencyHalt = async () => {
    try {
      const newState = await emergencyHalt("User manual kill switch");
      setOrchestratorState(newState);
      setShowHaltingConfirm(false);
    } catch (e) {
      console.error("Failed to emergency halt", e);
    }
  };

  const handleRiskChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = parseInt(e.target.value);
    setRiskValue(val);
    try {
      // Debouncing could be added here, but for now direct call
      await setRiskThrottle(val / 100);
    } catch (err) {
      console.error("Failed to set risk throttle", err);
    }
  };

  // Other actions (kept from original)
  const handleSwarm = async () => {
    await summonSwarm({ task: 'sector rotation diagnostics' });
  };

  const handleResearch = async () => {
    await requestResearch({ query: 'Top volatility catalysts within 24h' });
  };

  const isLive = orchestratorState?.is_live;
  const isEmergency = orchestratorState?.emergency_halt_active;
  const isWindingDown = orchestratorState?.is_winding_down;

  return (
    <div className="flex h-full flex-col rounded-lg border border-white/5 bg-slate-900/40 p-1" data-testid="action-dashboard">
      <div className="mb-2 flex items-center justify-between px-4 py-2 border-b border-white/5">
        <div className="flex items-center">
          <Terminal className="mr-2 h-4 w-4 text-emerald-500" />
          <h3 className="text-xs font-bold uppercase tracking-[0.2em] text-white/60">Command Deck</h3>
        </div>
        {isLive && (
          <div className="flex items-center space-x-2">
            <span className="relative flex h-3 w-3">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-3 w-3 bg-emerald-500"></span>
            </span>
            <span className="text-[10px] text-emerald-500 font-bold">LIVE</span>
          </div>
        )}
         {isWindingDown && !isEmergency && (
          <div className="flex items-center space-x-2">
             <span className="h-2 w-2 rounded-full bg-amber-500 animate-pulse" />
             <span className="text-[10px] text-amber-500 font-bold">WINDING DOWN</span>
          </div>
        )}
        {isEmergency && (
           <div className="flex items-center space-x-2">
             <span className="h-2 w-2 rounded-full bg-red-600 animate-pulse" />
             <span className="text-[10px] text-red-600 font-bold">HALTED</span>
           </div>
        )}
      </div>
      
      {/* Orchestrator Controls */}
      <div className="mx-2 mb-4 space-y-3 rounded bg-white/5 p-3">
        <div className="grid grid-cols-3 gap-2">
            <button
                onClick={handleEngageClick}
                disabled={isLive}
                className={`
                    flex flex-col items-center justify-center rounded p-2 transition-all
                    ${isLive 
                        ? 'bg-emerald-500/20 text-emerald-400 cursor-default border border-emerald-500/50 shadow-[0_0_10px_rgba(16,185,129,0.2)]' 
                        : 'bg-white/5 hover:bg-emerald-500/20 hover:text-emerald-400 text-slate-400 border border-transparent'
                    }
                    ${isEmergency ? 'animate-pulse ring-1 ring-emerald-500/50' : ''}
                `}
            >
                <Play className="h-5 w-5 mb-1" />
                <span className="text-[9px] font-bold">ENGAGE</span>
            </button>

            <button
                onClick={handleWindDown}
                disabled={!isLive || isWindingDown || isEmergency}
                className={`
                    flex flex-col items-center justify-center rounded p-2 transition-all
                    ${isWindingDown
                        ? 'bg-amber-500/20 text-amber-400 cursor-default border border-amber-500/50' 
                        : 'bg-white/5 hover:bg-amber-500/20 hover:text-amber-400 text-slate-400 border border-transparent'
                    }
                    ${!isLive || isEmergency ? 'opacity-30 cursor-not-allowed' : ''}
                `}
            >
               <Activity className="h-5 w-5 mb-1" />
               <span className="text-[9px] font-bold">SOFT STOP</span>
            </button>

            <button
                onClick={() => setShowHaltingConfirm(true)}
                disabled={isEmergency}
                 className={`
                    flex flex-col items-center justify-center rounded p-2 transition-all
                    ${isEmergency
                        ? 'bg-red-900/40 text-red-500 cursor-default border border-red-500' 
                        : 'bg-red-950/20 hover:bg-red-900/60 text-red-400 border border-red-900/50 hover:border-red-500'
                    }
                `}
            >
                <Power className="h-5 w-5 mb-1" />
                <span className="text-[9px] font-bold">KILL SWITCH</span>
            </button>
        </div>

        {/* Risk Throttle */}
        <div className="space-y-1">
            <div className="flex justify-between text-[10px] text-slate-400">
                <span>Risk Throttle</span>
                <span className={riskValue === 0 ? 'text-red-500 font-bold' : 'text-emerald-400'}>{riskValue}%</span>
            </div>
            <input 
                type="range" 
                min="0" 
                max="100" 
                value={riskValue} 
                onChange={handleRiskChange}
                className="w-full h-1.5 bg-slate-700 rounded-lg appearance-none cursor-pointer accent-emerald-500"
            />
        </div>
      </div>

      {/* Confirmation Modal for Kill Switch */}
      {showHaltingConfirm && (
        <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/80 rounded-lg p-4 backdrop-blur-sm">
            <div className="w-full max-w-sm rounded border border-red-500 bg-slate-900 p-4 shadow-[0_0_30px_rgba(239,68,68,0.3)]">
                <div className="flex items-center mb-3 text-red-500">
                    <ShieldAlert className="mr-2 h-6 w-6" />
                    <h3 className="font-bold text-lg">EMERGENCY HALT</h3>
                </div>
                <p className="mb-4 text-xs text-slate-300">
                    This will immediately disconnect all exchange feeds and freeze all trading activity. Manual intervention required to resume.
                </p>
                <div className="flex space-x-2">
                    <button 
                        onClick={() => setShowHaltingConfirm(false)}
                        className="flex-1 rounded bg-slate-700 py-2 text-xs font-bold hover:bg-slate-600"
                    >
                        CANCEL
                    </button>
                    <button 
                        onClick={handleEmergencyHalt}
                        className="flex-1 rounded bg-red-600 py-2 text-xs font-bold text-white hover:bg-red-700"
                    >
                        CONFIRM HALT
                    </button>
                </div>
            </div>
        </div>
      )}

      {/* Recovery Confirmation Modal - shown when re-engaging after emergency halt */}
      {showRecoveryConfirm && (
        <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/80 rounded-lg p-4 backdrop-blur-sm">
            <div className="w-full max-w-sm rounded border border-amber-500 bg-slate-900 p-4 shadow-[0_0_30px_rgba(245,158,11,0.3)]">
                <div className="flex items-center mb-3 text-amber-500">
                    <ShieldAlert className="mr-2 h-6 w-6" />
                    <h3 className="font-bold text-lg">RECOVERY MODE</h3>
                </div>
                <div className="mb-4 p-3 rounded bg-red-950/50 border border-red-500/30">
                    <p className="text-xs text-red-400 font-bold mb-1">⚠️ KILL SWITCH WAS TRIGGERED</p>
                    <p className="text-[10px] text-slate-400">
                        Reason: {orchestratorState?.emergency_halt_reason || 'Unknown'}
                    </p>
                </div>
                <p className="mb-4 text-xs text-slate-300">
                    Before re-engaging, please verify:
                </p>
                <ul className="mb-4 text-[10px] text-slate-400 space-y-1 list-disc list-inside">
                    <li>All exchange connections are stable</li>
                    <li>Account balances are correct</li>
                    <li>Risk parameters are configured properly</li>
                    <li>No pending orders require manual review</li>
                </ul>
                <div className="flex space-x-2">
                    <button 
                        onClick={() => setShowRecoveryConfirm(false)}
                        className="flex-1 rounded bg-slate-700 py-2 text-xs font-bold hover:bg-slate-600"
                    >
                        CANCEL
                    </button>
                    <button 
                        onClick={handleEngage}
                        className="flex-1 rounded bg-emerald-600 py-2 text-xs font-bold text-white hover:bg-emerald-700"
                    >
                        CONFIRM RE-ENGAGE
                    </button>
                </div>
            </div>
        </div>
      )}

      {/* Legacy/Other Actions */}
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

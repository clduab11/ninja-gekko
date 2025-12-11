import { useEffect, useState } from "react";
import {
  Loader2,
  Pause,
  Play,
  Rocket,
  Settings,
  ShieldCheck,
  SquareChartGantt,
  Zap,
  Menu,
  AlertTriangle,
} from "lucide-react";

import { useChatController } from "./hooks/useChatController";
import PersonaControls from "./components/panels/PersonaControls";
import ChatConversation from "./components/chat/ChatConversation";
import ChatComposer from "./components/chat/ChatComposer";
import ActionDashboard from "./components/panels/ActionDashboard";
import DiagnosticsPanel from "./components/panels/DiagnosticsPanel";
import MarketRadar from "./components/panels/MarketRadar";
import { Modal } from "./components/ui/Modal";
import HeaderMetrics from "./components/ui/HeaderMetrics";
import { engage, emergencyHalt, getOrchestratorState } from "./services/api";
import { OrchestratorState } from "./types";

function App() {
  const {
    messages,
    persona,
    diagnostics,
    isSending,
    isPersonaLoading,
    sendMessage,
    savePersona,
  } = useChatController();
  const [duration, setDuration] = useState(4);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [orchestratorState, setOrchestratorState] =
    useState<OrchestratorState | null>(null);

  // Poll orchestrator state
  useEffect(() => {
    const fetchState = async () => {
      try {
        const state = await getOrchestratorState();
        setOrchestratorState(state);
      } catch (e) {
        console.error("Failed to fetch orchestrator state:", e);
      }
    };

    fetchState();
    const interval = setInterval(fetchState, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleHeaderEngage = async () => {
    try {
      const newState = await engage();
      setOrchestratorState(newState);
    } catch (e) {
      console.error("Failed to engage:", e);
    }
  };

  const handleHeaderHalt = async () => {
    try {
      const newState = await emergencyHalt("Header kill switch activated");
      setOrchestratorState(newState);
    } catch (e) {
      console.error("Failed to halt:", e);
    }
  };

  const isLive = orchestratorState?.is_live;
  const isEmergency = orchestratorState?.emergency_halt_active;
  const isWindingDown = orchestratorState?.is_winding_down;

  // Determine status indicator color
  const getStatusIndicator = () => {
    if (isEmergency) return "bg-red-600 animate-pulse";
    if (isWindingDown) return "bg-amber-500 animate-pulse";
    if (isLive) return "bg-emerald-500 animate-pulse";
    return "bg-slate-600";
  };

  return (
    <div
      className="flex h-screen w-full flex-col bg-slate-950 text-slate-100 overflow-hidden font-sans selection:bg-emerald-500/30"
      data-testid="app-container"
    >
      <header
        className="flex h-12 shrink-0 items-center justify-between border-b border-white/5 bg-slate-950 px-4"
        role="banner"
        aria-label="Gordon Gekko Financial Terminal"
      >
        <div className="flex items-center gap-6">
          <div className="flex items-center gap-3">
            <div className="flex h-8 w-8 items-center justify-center rounded bg-emerald-500 text-slate-900 font-black text-xs tracking-tighter shadow-[0_0_15px_#10b981]">
              GG
            </div>
            <div>
              <h1 className="text-sm font-bold tracking-tight text-white uppercase">
                Gordon<span className="text-emerald-500">Gekko</span>
              </h1>
              <div className="text-[9px] font-medium uppercase tracking-[0.2em] text-slate-500">
                Financial Ninja
              </div>
            </div>
          </div>

          {/* Net Liq & Exposure Display */}
          <HeaderMetrics />

          {/* Compact Orchestration Controls */}
          <div
            className="flex items-center gap-3 rounded border border-white/5 bg-white/[0.02] px-3 py-1"
            role="toolbar"
            aria-label="Orchestration Controls"
          >
            <div className="flex items-center gap-2">
              <span className="text-[9px] uppercase tracking-wider text-slate-600 font-bold">
                Orchestrator
              </span>
              <div
                className={`h-1.5 w-1.5 rounded-full ${getStatusIndicator()}`}
              ></div>
              {isEmergency && (
                <AlertTriangle className="h-3 w-3 text-red-500" />
              )}
            </div>
            <div className="h-3 w-px bg-white/10" />
            <button
              onClick={handleHeaderEngage}
              disabled={isLive || isEmergency}
              className={`group flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider transition-colors ${
                isLive
                  ? "text-emerald-400"
                  : isEmergency
                  ? "text-slate-600 cursor-not-allowed opacity-50"
                  : "text-slate-400 hover:text-emerald-400"
              }`}
              data-testid="btn-resume-trading"
            >
              <div
                className={`flex h-4 w-4 items-center justify-center rounded-full border border-current ${
                  isLive ? "bg-emerald-500/20" : "group-hover:bg-emerald-500/20"
                }`}
              >
                <Play className="h-2 w-2" fill="currentColor" />
              </div>
              <span>{isLive ? "LIVE" : "Live"}</span>
            </button>
            <button
              onClick={handleHeaderHalt}
              disabled={isEmergency}
              className={`group flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider transition-colors ${
                isEmergency
                  ? "text-red-500"
                  : "text-slate-400 hover:text-red-400"
              }`}
              data-testid="btn-pause-trading-header"
            >
              <div
                className={`flex h-4 w-4 items-center justify-center rounded-full border border-current ${
                  isEmergency ? "bg-red-500/20" : "group-hover:bg-red-500/20"
                }`}
              >
                <Pause className="h-2 w-2" fill="currentColor" />
              </div>
              <span>{isEmergency ? "HALTED" : "Kill"}</span>
            </button>
            <input
              type="range"
              min={1}
              max={24}
              value={duration}
              onChange={(event) => setDuration(Number(event.target.value))}
              className="h-1 w-12 accent-emerald-500 opacity-30 hover:opacity-100 transition-opacity"
              title={`Pause Duration: ${duration}h`}
            />
          </div>
        </div>

        <div className="flex items-center gap-4">
          <div className="hidden md:flex items-center gap-2 text-[10px] font-medium uppercase tracking-wider text-slate-600">
            <ShieldCheck className="h-3 w-3 text-emerald-900" />
            <span>Secure Connection</span>
          </div>

          <button
            onClick={() => setIsSettingsOpen(true)}
            className="flex items-center gap-2 rounded border border-white/10 bg-white/5 px-2 py-1 text-[10px] text-slate-400 hover:bg-white/10 hover:text-white transition-colors"
            aria-label="Open Persona Settings"
          >
            <Settings className="h-3 w-3" />
            <span>Config</span>
          </button>
        </div>
      </header>

      {/* COMMAND CENTER GRID - New Layout */}
      {/* Top Row: Market Radar (Dominant) + Action Dashboard */}
      {/* Bottom Row: Chat (Terminal) */}
      <main
        className="grid flex-1 grid-rows-[65%_35%] gap-px bg-slate-900 overflow-hidden"
        role="main"
      >
        {/* TOP SEC: Intelligence & Controls */}
        <div className="grid grid-cols-[1fr_320px] gap-px bg-slate-950">
          {/* Market Radar */}
          <section className="bg-slate-950 p-1 overflow-hidden relative">
            <div className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-emerald-500/20 to-transparent opacity-50"></div>
            <MarketRadar />
          </section>

          {/* Control Deck */}
          <section className="bg-slate-950 p-1 border-l border-white/5">
            <ActionDashboard />
          </section>
        </div>

        {/* BOTTOM SEC: Communication Channel */}
        <section
          className="flex flex-col bg-slate-950 border-t border-white/5 relative"
          aria-label="Direct Communication Line"
        >
          <div className="absolute left-0 top-0 flex items-center gap-2 bg-slate-900 px-3 py-1 rounded-br border-r border-b border-white/5 z-10">
            <div className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse"></div>
            <span className="text-[9px] font-bold uppercase tracking-widest text-slate-400">
              Direct Uplink
            </span>
          </div>

          <div className="flex-1 overflow-hidden flex flex-col relative pt-6">
            <div className="absolute inset-0 bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-[0.03] pointer-events-none mix-blend-overlay" />
            <ChatConversation messages={messages} />
          </div>

          <ChatComposer disabled={isSending} onSend={sendMessage} />
        </section>
      </main>

      {/* FOOTER */}
      <footer className="flex h-6 shrink-0 items-center justify-between border-t border-white/5 bg-black px-4 text-[9px] text-slate-600 uppercase tracking-widest font-mono">
        <div className="flex items-center gap-4">
          <span className="text-emerald-900 font-bold">Ninja Gekko v2.1.0</span>
          <span className="flex items-center gap-1 text-emerald-500/40">
            <Rocket className="h-3 w-3" /> System Nominal
          </span>
        </div>
        <div className="flex items-center gap-4">
          <span>Memory: 14%</span>
          <span>Latency: 8ms</span>
        </div>
      </footer>

      {/* MODALS */}
      <Modal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        title="System Configuration"
      >
        <PersonaControls
          persona={persona}
          onSave={savePersona}
          isLoading={isPersonaLoading}
        />
        <div className="mt-6 border-t border-white/10 pt-4">
          <DiagnosticsPanel diagnostics={diagnostics} />
        </div>
      </Modal>
    </div>
  );
}

export default App;

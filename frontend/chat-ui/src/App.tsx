import { useMemo, useState } from 'react';
import { Loader2, Pause, Play, Rocket, Sparkles, SquareChartGantt } from 'lucide-react';

import { useChatController } from './hooks/useChatController';
import PersonaControls from './components/panels/PersonaControls';
import InsightsPanel from './components/panels/InsightsPanel';
import ChatConversation from './components/chat/ChatConversation';
import ChatComposer from './components/chat/ChatComposer';
import ActionDashboard from './components/panels/ActionDashboard';
import DiagnosticsPanel from './components/panels/DiagnosticsPanel';

function App() {
  const { messages, persona, diagnostics, isSending, isPersonaLoading, sendMessage, savePersona } =
    useChatController();
  const [duration, setDuration] = useState(4);

  const personaLabel = useMemo(() => `${persona.tone} · ${persona.style} · ${persona.mood}`, [persona]);

  return (
    <div className="min-h-screen bg-background text-white" data-testid="app-container">
      <header 
        className="flex items-center justify-between px-10 py-6 border-b border-border/60"
        role="banner"
        aria-label="Ninja Gekko application header"
      >
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Talk to Gordon</h1>
          <p className="text-sm text-white/70">Institutional-grade agentic control for Ninja Gekko</p>
        </div>
        <div 
          className="flex items-center gap-3 text-sm text-white/70"
          data-testid="indicator-persona"
          aria-label={`Current persona: ${personaLabel}`}
        >
          <Sparkles className="h-4 w-4 text-accent" aria-hidden="true" />
          <span>Persona: {personaLabel}</span>
          {isPersonaLoading ? (
            <Loader2 
              className="h-4 w-4 animate-spin text-accent" 
              data-testid="indicator-loading"
              aria-label="Loading persona"
            />
          ) : null}
        </div>
      </header>

      <main 
        className="grid grid-cols-[2.1fr_1.2fr] gap-6 px-10 py-8"
        role="main"
        data-testid="main-content"
        aria-label="Trading orchestration interface"
      >
        <section className="flex flex-col gap-4" data-testid="orchestration-section">
          <div 
            className="flex items-center justify-between rounded-xl border border-border/40 bg-panel/80 px-6 py-4"
            data-testid="orchestration-controls"
            role="region"
            aria-label="Live orchestration controls"
          >
            <div>
              <h2 className="text-lg font-semibold">Live Orchestration</h2>
              <p className="text-sm text-white/60">Control trading automations, research swarms, and MPC flows.</p>
            </div>
            <div className="flex items-center gap-3 text-sm">
              <button 
                className="flex items-center gap-2 rounded-lg border border-accentSoft/60 px-3 py-2 font-medium text-accent"
                data-testid="btn-resume-trading"
                role="button"
                aria-label="Resume trading operations"
              >
                <Play className="h-4 w-4" aria-hidden="true" /> Resume
              </button>
              <button 
                className="flex items-center gap-2 rounded-lg border border-border px-3 py-2 font-medium text-white/80 hover:border-accent"
                data-testid="btn-pause-trading-header"
                role="button"
                aria-label={`Pause trading for ${duration} hours`}
              >
                <Pause className="h-4 w-4" aria-hidden="true" /> Pause {duration}h
              </button>
              <input
                type="range"
                min={1}
                max={24}
                value={duration}
                onChange={(event) => setDuration(Number(event.target.value))}
                className="h-1 w-32 accent-accent"
                data-testid="slider-pause-duration"
                aria-label="Pause duration in hours"
                aria-valuemin={1}
                aria-valuemax={24}
                aria-valuenow={duration}
              />
            </div>
          </div>

          <div className="flex h-[65vh] gap-4">
            <div 
              className="flex w-full flex-col rounded-xl border border-border/40 bg-panel/80"
              data-testid="chat-panel"
              role="region"
              aria-label="Chat conversation with Gordon"
            >
              <div className="flex items-center justify-between border-b border-border/60 px-6 py-4">
                <div>
                  <h2 className="text-lg font-semibold">Conversation</h2>
                  <p className="text-xs uppercase tracking-[0.28em] text-white/40">Memory · Citations · Control</p>
                </div>
                <span 
                  className="flex items-center gap-2 rounded-full bg-accentSoft/20 px-3 py-1 text-xs text-accent"
                  data-testid="indicator-autonomous-mode"
                  role="status"
                  aria-label="Autonomous mode active"
                >
                  <SquareChartGantt className="h-4 w-4" aria-hidden="true" /> Autonomous Mode
                </span>
              </div>
              <ChatConversation messages={messages} />
              <ChatComposer disabled={isSending} onSend={sendMessage} />
            </div>

            <div className="flex w-[28rem] flex-col gap-4" data-testid="side-panels">
              <PersonaControls persona={persona} onSave={savePersona} isLoading={isPersonaLoading} />
              <DiagnosticsPanel diagnostics={diagnostics} />
              <ActionDashboard />
            </div>
          </div>
        </section>

        <aside 
          className="flex flex-col gap-4"
          role="complementary"
          aria-label="Trading insights and intelligence"
          data-testid="insights-aside"
        >
          <InsightsPanel />
        </aside>
      </main>

      <footer 
        className="flex items-center justify-between px-10 py-4 text-xs text-white/40"
        role="contentinfo"
        data-testid="app-footer"
      >
        <span>© {new Date().getFullYear()} Ninja Gekko · Agentic Trading Intelligence</span>
        <span 
          className="flex items-center gap-2"
          data-testid="indicator-mcp-status"
          role="status"
          aria-label="MCP Mesh connection active"
        >
          <Rocket className="h-4 w-4" aria-hidden="true" /> MCP Mesh Active
        </span>
      </footer>
    </div>
  );
}

export default App;

import { useQuery } from '@tanstack/react-query';
import { BarChart3, BrainCircuit, Hand } from 'lucide-react';

import { fetchAccountSnapshot, fetchNews, pauseTrading, requestResearch, summonSwarm } from '../../services/api';
import { AccountSnapshot, NewsHeadline } from '../../types';

const ActionDashboard = () => {
  const { data: snapshot } = useQuery({ queryKey: ['account-snapshot'], queryFn: fetchAccountSnapshot });
  const { data: news } = useQuery({ queryKey: ['news'], queryFn: fetchNews });

  const handlePause = async () => {
    await pauseTrading({ duration_hours: 6 });
  };

  const handleSwarm = async () => {
    await summonSwarm({ task: 'sector rotation diagnostics' });
  };

  const handleResearch = async () => {
    await requestResearch({ query: 'Top volatility catalysts within 24h' });
  };

  return (
    <section className="rounded-xl border border-border/60 bg-panel/80 p-4 text-sm">
      <header className="mb-3 text-xs uppercase tracking-[0.35em] text-white/40">Control Center</header>
      <div className="grid gap-3">
        <button
          onClick={handlePause}
          className="flex items-center justify-between rounded-lg border border-border/60 bg-panel px-3 py-2 text-left text-sm hover:border-accent"
        >
          <span className="flex items-center gap-2 text-white/80">
            <Hand className="h-4 w-4 text-accent" /> Pause Trading (6h)
          </span>
          <span className="text-xs text-white/40">OANDA · Coinbase · Binance.us</span>
        </button>
        <button
          onClick={handleSwarm}
          className="flex items-center justify-between rounded-lg border border-border/60 bg-panel px-3 py-2 text-left text-sm hover:border-accent"
        >
          <span className="flex items-center gap-2 text-white/80">
            <BrainCircuit className="h-4 w-4 text-accent" /> Summon Swarm
          </span>
          <span className="text-xs text-white/40">Strategy · Diagnostics · Retraining</span>
        </button>
        <button
          onClick={handleResearch}
          className="flex items-center justify-between rounded-lg border border-border/60 bg-panel px-3 py-2 text-left text-sm hover:border-accent"
        >
          <span className="flex items-center gap-2 text-white/80">
            <BarChart3 className="h-4 w-4 text-accent" /> Deep Research Pulse
          </span>
          <span className="text-xs text-white/40">Perplexity Finance · Sonar</span>
        </button>
      </div>

      <div className="mt-4 space-y-3">
        <SnapshotCard snapshot={snapshot} />
        <NewsCard headlines={news} />
      </div>
    </section>
  );
};

const SnapshotCard = ({ snapshot }: { snapshot?: AccountSnapshot }) => {
  if (!snapshot) return null;
  return (
    <article className="rounded-lg border border-border/60 bg-panel px-3 py-3">
      <header className="text-xs uppercase tracking-[0.35em] text-white/40">Account Snapshot</header>
      <div className="mt-2 text-sm">
        <p className="text-lg font-semibold text-accent">
          ${snapshot.total_equity.toLocaleString(undefined, { maximumFractionDigits: 0 })}
        </p>
        <p className="text-xs text-white/50">Net exposure: {(snapshot.net_exposure * 100).toFixed(1)}%</p>
      </div>
      <ul className="mt-3 space-y-1 text-xs text-white/60">
        {snapshot.brokers.map((broker) => (
          <li key={broker.broker} className="flex items-center justify-between">
            <span>{broker.broker}</span>
            <span>
              ${broker.balance.toLocaleString(undefined, { maximumFractionDigits: 0 })} · {broker.open_positions} positions
            </span>
          </li>
        ))}
      </ul>
    </article>
  );
};

const NewsCard = ({ headlines }: { headlines?: NewsHeadline[] }) => {
  if (!headlines?.length) return null;
  return (
    <article className="rounded-lg border border-border/60 bg-panel px-3 py-3">
      <header className="text-xs uppercase tracking-[0.35em] text-white/40">Realtime Signals</header>
      <ul className="mt-2 space-y-2 text-xs">
        {headlines.map((item) => (
          <li key={item.id} className="leading-relaxed text-white/80">
            <a href={item.url} target="_blank" rel="noreferrer" className="hover:text-accent">
              {item.title}
            </a>
            <div className="text-[10px] uppercase tracking-[0.3em] text-white/30">
              {item.source} · {new Date(item.published_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
            </div>
          </li>
        ))}
      </ul>
    </article>
  );
};

export default ActionDashboard;

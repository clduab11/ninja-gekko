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
    <section 
      className="rounded-xl border border-border/60 bg-panel/80 p-4 text-sm"
      data-testid="action-dashboard"
      role="region"
      aria-label="Trading control center"
    >
      <header className="mb-3 text-xs uppercase tracking-[0.35em] text-white/40">Control Center</header>
      <div className="grid gap-3" role="group" aria-label="Trading action buttons">
        <button
          onClick={handlePause}
          className="flex items-center justify-between rounded-lg border border-border/60 bg-panel px-3 py-2 text-left text-sm hover:border-accent"
          data-testid="btn-pause-trading"
          role="button"
          aria-label="Pause trading for 6 hours across all exchanges"
        >
          <span className="flex items-center gap-2 text-white/80">
            <Hand className="h-4 w-4 text-accent" aria-hidden="true" /> Pause Trading (6h)
          </span>
          <span className="text-xs text-white/40">OANDA · Coinbase · Binance.us</span>
        </button>
        <button
          onClick={handleSwarm}
          className="flex items-center justify-between rounded-lg border border-border/60 bg-panel px-3 py-2 text-left text-sm hover:border-accent"
          data-testid="btn-summon-swarm"
          role="button"
          aria-label="Summon AI research swarm for strategy diagnostics"
        >
          <span className="flex items-center gap-2 text-white/80">
            <BrainCircuit className="h-4 w-4 text-accent" aria-hidden="true" /> Summon Swarm
          </span>
          <span className="text-xs text-white/40">Strategy · Diagnostics · Retraining</span>
        </button>
        <button
          onClick={handleResearch}
          className="flex items-center justify-between rounded-lg border border-border/60 bg-panel px-3 py-2 text-left text-sm hover:border-accent"
          data-testid="btn-research-pulse"
          role="button"
          aria-label="Request deep research analysis on market volatility"
        >
          <span className="flex items-center gap-2 text-white/80">
            <BarChart3 className="h-4 w-4 text-accent" aria-hidden="true" /> Deep Research Pulse
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
    <article 
      className="rounded-lg border border-border/60 bg-panel px-3 py-3"
      data-testid="card-account-snapshot"
      role="region"
      aria-label="Account snapshot with equity and exposure"
    >
      <header className="text-xs uppercase tracking-[0.35em] text-white/40">Account Snapshot</header>
      <div className="mt-2 text-sm">
        <p 
          className="text-lg font-semibold text-accent"
          data-testid="metric-total-equity"
          aria-label={`Total equity: $${snapshot.total_equity.toLocaleString(undefined, { maximumFractionDigits: 0 })}`}
        >
          ${snapshot.total_equity.toLocaleString(undefined, { maximumFractionDigits: 0 })}
        </p>
        <p 
          className="text-xs text-white/50"
          data-testid="metric-net-exposure"
          aria-label={`Net exposure: ${(snapshot.net_exposure * 100).toFixed(1)}%`}
        >
          Net exposure: {(snapshot.net_exposure * 100).toFixed(1)}%
        </p>
      </div>
      <ul 
        className="mt-3 space-y-1 text-xs text-white/60"
        data-testid="broker-list"
        role="list"
        aria-label="Broker account balances"
      >
        {snapshot.brokers.map((broker, index) => (
          <li 
            key={broker.broker} 
            className="flex items-center justify-between"
            data-testid={`broker-item-${index}`}
            role="listitem"
          >
            <span data-testid={`broker-name-${index}`}>{broker.broker}</span>
            <span data-testid={`broker-balance-${index}`}>
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
    <article 
      className="rounded-lg border border-border/60 bg-panel px-3 py-3"
      data-testid="card-realtime-signals"
      role="region"
      aria-label="Realtime market signals and news"
    >
      <header className="text-xs uppercase tracking-[0.35em] text-white/40">Realtime Signals</header>
      <ul 
        className="mt-2 space-y-2 text-xs"
        data-testid="news-list"
        role="list"
        aria-label="Latest market news headlines"
        aria-live="polite"
      >
        {headlines.map((item, index) => (
          <li 
            key={item.id} 
            className="leading-relaxed text-white/80"
            data-testid={`news-item-${index}`}
            role="listitem"
          >
            <a 
              href={item.url} 
              target="_blank" 
              rel="noreferrer" 
              className="hover:text-accent"
              data-testid={`news-link-${index}`}
              aria-label={`Read article: ${item.title}`}
            >
              {item.title}
            </a>
            <div 
              className="text-[10px] uppercase tracking-[0.3em] text-white/30"
              data-testid={`news-meta-${index}`}
            >
              {item.source} · {new Date(item.published_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
            </div>
          </li>
        ))}
      </ul>
    </article>
  );
};

export default ActionDashboard;

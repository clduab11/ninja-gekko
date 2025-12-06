import { useQuery } from '@tanstack/react-query';
import { ActivitySquare, Database, FileSearch, GlobeLock, LineChart } from 'lucide-react';

import { fetchAccountSnapshot, fetchNews } from '../../services/api';

const InsightsPanel = () => {
  const { data: snapshot } = useQuery({ queryKey: ['account-snapshot'], queryFn: fetchAccountSnapshot });
  const { data: news } = useQuery({ queryKey: ['news'], queryFn: fetchNews });

  return (
    <section 
      className="flex h-full flex-col gap-4 rounded-xl border border-border/60 bg-panel/80 p-6 text-sm"
      data-testid="insights-panel"
      role="region"
      aria-label="Realtime trading intelligence"
    >
      <header>
        <h2 className="text-lg font-semibold">Realtime Intel</h2>
        <p className="text-xs text-white/50">Positions · Risk · Rationale · Research</p>
      </header>

      <div 
        className="grid gap-3"
        role="group"
        aria-label="Trading insight metrics"
      >
        <InsightTile
          testId="insight-tile-exposure"
          icon={<LineChart className="h-4 w-4" />}
          title="Current Exposure"
          value={snapshot ? `${(snapshot.net_exposure * 100).toFixed(1)}% net` : 'Loading...'}
          description="Cross-venue leverage, hedges, and directional skew"
        />
        <InsightTile
          testId="insight-tile-automation"
          icon={<ActivitySquare className="h-4 w-4" />}
          title="Automation"
          value="Swarm + Precision"
          description="Agentic mesh orchestrating micro + macro plays"
        />
        <InsightTile
          testId="insight-tile-data-streams"
          icon={<Database className="h-4 w-4" />}
          title="Data Streams"
          value="Perplexity Finance, Sonar, MCP Mesh"
          description="Streaming transcripts, order books, compliance logs"
        />
        <InsightTile
          testId="insight-tile-security"
          icon={<GlobeLock className="h-4 w-4" />}
          title="Security Mode"
          value="Zero-trust · Encrypted"
          description="Playwright & Filesystem MCP with admin lattice"
        />
      </div>

      <div 
        className="rounded-xl border border-border/60 bg-panel px-4 py-3 text-xs text-white/70"
        data-testid="market-radar"
        role="region"
        aria-label="Market news and signals radar"
      >
        <div className="mb-2 flex items-center gap-2 text-[11px] uppercase tracking-[0.4em] text-white/30">
          <FileSearch className="h-4 w-4" aria-hidden="true" /> Market Radar
        </div>
        <ul 
          className="space-y-2"
          role="list"
          aria-label="Latest market radar signals"
          aria-live="polite"
        >
          {news?.map((item, index) => (
            <li 
              key={item.id}
              data-testid={`radar-item-${index}`}
              role="listitem"
            >
              <a 
                href={item.url} 
                target="_blank" 
                rel="noreferrer" 
                className="font-medium text-white/80 hover:text-accent"
                data-testid={`radar-link-${index}`}
                aria-label={`Read: ${item.title}`}
              >
                {item.title}
              </a>
              <div 
                className="text-[10px] uppercase tracking-[0.3em] text-white/30"
                data-testid={`radar-meta-${index}`}
              >
                {item.source} · {new Date(item.published_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
              </div>
            </li>
          )) || <li data-testid="radar-loading">Loading sonar + perplexity feeds...</li>}
        </ul>
      </div>
    </section>
  );
};

interface InsightTileProps {
  testId: string;
  icon: JSX.Element;
  title: string;
  value: string;
  description: string;
}

const InsightTile = ({ testId, icon, title, value, description }: InsightTileProps) => (
  <article 
    className="rounded-xl border border-border/60 bg-panel px-4 py-3"
    data-testid={testId}
    role="article"
    aria-label={`${title}: ${value}`}
  >
    <div className="flex items-center gap-2 text-xs uppercase tracking-[0.35em] text-white/40">
      <span className="text-accent" aria-hidden="true">{icon}</span>
      {title}
    </div>
    <p 
      className="mt-2 text-lg font-semibold text-white/90"
      data-testid={`${testId}-value`}
    >
      {value}
    </p>
    <p 
      className="text-[13px] leading-relaxed text-white/60"
      data-testid={`${testId}-description`}
    >
      {description}
    </p>
  </article>
);

export default InsightsPanel;

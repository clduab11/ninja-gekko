import React, { useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Area, AreaChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';
import { Activity, Globe, Newspaper, Radio, Zap } from 'lucide-react';
import { fetchAggregateAccount } from '../../services/api';
import { useChartData } from '../../hooks/useChartData';
import { useIntelWebSocket } from '../../hooks/useIntelWebSocket';

const EXCHANGES = {
  KRAKEN: { 
    id: 'KRAKEN', 
    name: 'Kraken', 
    type: 'SPOT', 
    pairs: [
      { symbol: 'XBT/USD', name: 'Bitcoin', category: 'crypto' },
      { symbol: 'ETH/USD', name: 'Ethereum', category: 'crypto' },
      { symbol: 'SOL/USD', name: 'Solana', category: 'crypto' },
      { symbol: 'DOGE/USD', name: 'Dogecoin', category: 'crypto' },
      { symbol: 'USDT/USD', name: 'Tether', category: 'crypto' }
    ] 
  },
  BINANCE: { 
    id: 'BINANCE', 
    name: 'Binance.US', 
    type: 'SPOT', 
    pairs: [
      { symbol: 'BTCUSD', name: 'Bitcoin', category: 'crypto' },
      { symbol: 'ETHUSD', name: 'Ethereum', category: 'crypto' },
      { symbol: 'SOLUSD', name: 'Solana', category: 'crypto' },
      { symbol: 'BNBUSD', name: 'Binance Coin', category: 'crypto' },
      { symbol: 'ADAUSD', name: 'Cardano', category: 'crypto' }
    ] 
  },
  OANDA: { 
    id: 'OANDA', 
    name: 'OANDA', 
    type: 'FX/CFD', 
    pairs: [
      { symbol: 'EUR/USD', name: 'Euro', category: 'forex' },
      { symbol: 'GBP/USD', name: 'British Pound', category: 'forex' },
      { symbol: 'USD/JPY', name: 'Japanese Yen', category: 'forex' },
      { symbol: 'USD/CAD', name: 'Canadian Dollar', category: 'forex' },
      { symbol: 'AUD/USD', name: 'Aussie Dollar', category: 'forex' },
      { symbol: 'BTC/USD', name: 'Bitcoin (CFD)', category: 'cfd' },
      { symbol: 'ETH/USD', name: 'Ethereum (CFD)', category: 'cfd' }
    ] 
  }
};

const MarketRadar = () => {
  const [activeExchange, setActiveExchange] = React.useState<keyof typeof EXCHANGES>('KRAKEN');
  const [symbol, setSymbol] = React.useState(EXCHANGES['KRAKEN'].pairs[0].symbol);
  const [timeframe, setTimeframe] = React.useState('15m');


  const { data: aggregateAccount } = useQuery({ queryKey: ['aggregate-account'], queryFn: fetchAggregateAccount });
  
  // Use custom hook for chart data
  const { data: historicalData, isLoading: isChartLoading } = useChartData(symbol, timeframe);
  
  // Use Intel Stream hook
  const { items: intelItems, isConnected: isIntelConnected } = useIntelWebSocket();

  // State for live price updates
  const [livePrice, setLivePrice] = React.useState<number | null>(null);

  // WebSocket for live updates (Keep simple local state for header price for now, hook handles chart mostly)
  React.useEffect(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/api/v1/ws?symbols=${symbol}`;
    const ws = new WebSocket(wsUrl);

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.type === 'market_data' && msg.data?.symbol === symbol) {
            setLivePrice(msg.data.price);
        }
      } catch (e) {
        console.error('WS Parse Error', e);
      }
    };

    return () => {
      ws.close();
    };
  }, [symbol]);

  const chartData = useMemo(() => {
    if (!historicalData) return [];
    
    return historicalData.map(d => ({
      time: new Date(d.timestamp).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'}),
      value: d.price,
      original: d
    }));
  }, [historicalData]);

  const currentPrice = livePrice ?? (historicalData && historicalData.length > 0 ? historicalData[historicalData.length - 1].price : 0);
  const prevPrice = historicalData && historicalData.length > 1 ? historicalData[historicalData.length - 2].price : currentPrice;
  const isUp = currentPrice >= prevPrice;

  const handleResearch = () => {
    // In a real implementation, this would trigger the agent
    console.log(`Triggering deep research for ${symbol}`);
    // Dispatch a custom event that Chat component listens to, or just open a new window for now
    // window.open(`https://www.perplexity.ai/search?q=Analyze ${symbol} price action and future outlook`, '_blank');
  };

  return (
    <div className="flex h-full flex-col bg-slate-900/50 p-4" data-testid="market-radar">
      {/* Header / Stats Row */}
      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-emerald-500/10 text-emerald-500">
            <Radio className="h-4 w-4 animate-pulse" />
          </div>
          <div>
            <h2 className="text-sm font-bold uppercase tracking-widest text-slate-100">Market Radar</h2>
            <div className="flex items-center gap-2 text-[10px] text-emerald-400">
              <span className="relative flex h-2 w-2">
                <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75"></span>
                <span className="relative inline-flex h-2 w-2 rounded-full bg-emerald-500"></span>
              </span>
              MULTI-EXCHANGE FEED ACTIVE
            </div>
          </div>

          <div className="ml-8 flex gap-1 rounded-lg bg-slate-800/50 p-1">
            {(Object.keys(EXCHANGES) as Array<keyof typeof EXCHANGES>).map(ex => (
                <button
                    key={ex}
                    onClick={() => {
                        setActiveExchange(ex);
                        setSymbol(EXCHANGES[ex].pairs[0].symbol);
                    }}
                    className={`px-3 py-1 text-[10px] font-bold transition-all rounded ${activeExchange === ex ? 'bg-emerald-500 text-slate-900 shadow-lg shadow-emerald-500/20' : 'text-slate-400 hover:text-slate-200'}`}
                >
                    {EXCHANGES[ex].name}
                </button>
            ))}
          </div>
        </div>

        {/* Portfolio Stats embedded in header */}
        <div className="flex gap-6 text-right">
             <div>
                <div className="text-[10px] uppercase tracking-wider text-slate-500">Net Liq</div>
                <div className="font-mono text-sm font-bold text-slate-200">
                    ${aggregateAccount?.total_net_liquidity?.toLocaleString() ?? '---,---'}
                </div>
             </div>
             <div>
                <div className="text-[10px] uppercase tracking-wider text-slate-500">Global Exposure</div>
                <div className="font-mono text-sm font-bold text-slate-200">
                    $1,240,500.00
                </div>
             </div>
        </div>
      </div>

      {/* Main Content Grid: Watchlist | Chart | News */}
      <div className="grid flex-1 grid-cols-[240px_1fr_300px] gap-4 overflow-hidden">
        
        {/* Left: Watchlist */}
        <div className="flex flex-col rounded-lg border border-white/5 bg-slate-950/50">
            <div className="flex items-center justify-between border-b border-white/5 px-3 py-3 bg-white/5">
                <span className="text-[10px] font-bold uppercase tracking-wider text-slate-400">
                    {EXCHANGES[activeExchange].type} PAIRS
                </span>
                <Activity className="h-3 w-3 text-slate-500" />
            </div>
            <div className="flex-1 overflow-y-auto p-1 scrollbar-thin scrollbar-track-transparent scrollbar-thumb-white/10">
                <div className="space-y-0.5">
                    {EXCHANGES[activeExchange].pairs.map((pair) => (
                        <button
                            key={pair.symbol}
                            onClick={() => setSymbol(pair.symbol)}
                            className={`w-full group flex flex-col gap-0.5 rounded px-3 py-2 text-left transition-all ${symbol === pair.symbol ? 'bg-white/10 border-l-2 border-emerald-500' : 'hover:bg-white/5 border-l-2 border-transparent'}`}
                        >
                            <div className="flex items-center justify-between">
                                <span className={`text-xs font-bold ${symbol === pair.symbol ? 'text-emerald-400' : 'text-slate-200'}`}>
                                    {pair.symbol}
                                    {pair.category && (
                                      <span
                                        className={`ml-2 rounded px-1 py-0.5 text-[9px] font-semibold ${
                                          pair.category === 'forex'
                                            ? 'bg-blue-800/40 text-blue-300 border border-blue-600/30'
                                            : pair.category === 'cfd'
                                            ? 'bg-purple-800/40 text-purple-300 border border-purple-600/30'
                                            : pair.category === 'crypto'
                                            ? 'bg-emerald-900/40 text-emerald-300 border border-emerald-600/30'
                                            : 'hidden'
                                        }`}
                                        aria-label={
                                          pair.category === 'forex'
                                            ? 'Forex'
                                            : pair.category === 'cfd'
                                            ? 'CFD'
                                            : pair.category === 'crypto'
                                            ? 'Crypto'
                                            : ''
                                        }
                                        title={
                                          pair.category === 'forex'
                                            ? 'Forex Pair'
                                            : pair.category === 'cfd'
                                            ? 'Contract for Difference'
                                            : pair.category === 'crypto'
                                            ? 'Cryptocurrency'
                                            : ''
                                        }
                                      >
                                        {pair.category === 'forex'
                                          ? 'FX'
                                          : pair.category === 'cfd'
                                          ? 'CFD'
                                          : pair.category === 'crypto'
                                          ? 'CRYPTO'
                                          : ''}
                                      </span>
                                    )}
                                </span>
                                {symbol === pair.symbol && <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse"></span>}
                            </div>
                            <span className="text-[10px] text-slate-500 group-hover:text-slate-400">{pair.name}</span>
                        </button>
                    ))}
                </div>
            </div>
        </div>

        {/* Center: Main Chart Area */}
        <div className="relative flex flex-col rounded-lg border border-white/5 bg-slate-950/50 p-4">
            {/* Chart Header Overlay */}
            <div className="absolute left-4 top-4 z-10 flex items-center gap-4">
                <div>
                    <h3 className="text-xl font-black tracking-tight text-white">{symbol}</h3>
                    <div className={`flex items-center gap-2 text-sm font-mono font-bold ${isUp ? 'text-emerald-400' : 'text-red-400'}`}>
                        ${currentPrice.toLocaleString(undefined, {minimumFractionDigits: 2})}
                        <span className="text-[10px] opacity-75">
                            {isUp ? '+' : ''}{((currentPrice - prevPrice) / prevPrice * 100).toFixed(2)}%
                        </span>
                    </div>
                </div>
                
                {/* Agentic Control */}
                <button 
                    onClick={handleResearch}
                    className="flex items-center gap-1.5 rounded-full bg-indigo-500/20 px-3 py-1 text-[10px] font-bold text-indigo-300 hover:bg-indigo-500/30 hover:text-indigo-200 border border-indigo-500/30 transition-all"
                    title="Launch Autonomous Research Agent"
                >
                    <Globe className="h-3 w-3" />
                    DEEP ANALYZE
                </button>
            </div>

            <div className="absolute right-4 top-4 z-10 flex gap-2">
                {['15m', '1H', '4H', '1D'].map(tf => (
                    <button 
                        key={tf} 
                        onClick={() => setTimeframe(tf)}
                        className={`rounded px-2 py-1 text-[10px] font-bold hover:bg-white/5 hover:text-slate-300 ${timeframe === tf ? 'bg-white/10 text-emerald-400' : 'text-slate-500'}`}
                    >
                        {tf}
                    </button>
                ))}
            </div>
            
            <div className="mt-12 h-full w-full min-h-[200px]">
                {isChartLoading ? (
                    <div className="flex h-full w-full items-center justify-center text-slate-500">
                        <div className="flex flex-col items-center gap-2">
                            <Activity className="h-6 w-6 animate-bounce text-emerald-500" />
                            <span>Acquiring signal...</span>
                        </div>
                    </div>
                ) : (
                <ResponsiveContainer width="100%" height="100%">
                    <AreaChart data={chartData}>
                        <defs>
                            <linearGradient id="colorValue" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="5%" stopColor={isUp ? "#10b981" : "#ef4444"} stopOpacity={0.3}/>
                                <stop offset="95%" stopColor={isUp ? "#10b981" : "#ef4444"} stopOpacity={0}/>
                            </linearGradient>
                        </defs>
                        <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" vertical={false} />
                        <XAxis 
                            dataKey="time" 
                            stroke="#475569" 
                            tick={{fill: '#475569', fontSize: 10}} 
                            tickLine={false}
                            axisLine={false}
                            interval="preserveStartEnd"
                        />
                        <YAxis 
                            orientation="right" 
                            stroke="#475569" 
                            tick={{fill: '#475569', fontSize: 10}} 
                            tickLine={false} 
                            axisLine={false}
                            domain={['auto', 'auto']}
                        />
                        <Tooltip 
                            contentStyle={{backgroundColor: '#0f172a', borderColor: '#1e293b', fontSize: '12px'}}
                            itemStyle={{color: '#e2e8f0'}}
                        />
                        <Area 
                            type="monotone" 
                            dataKey="value" 
                            stroke={isUp ? "#10b981" : "#ef4444"} 
                            strokeWidth={2}
                            fillOpacity={1} 
                            fill="url(#colorValue)" 
                        />
                    </AreaChart>
                </ResponsiveContainer>
                )}
            </div>
        </div>

        {/* Right: Intel Stream Feed */}
        <div className="flex flex-col rounded-lg border border-white/5 bg-slate-950/50">
            <div className="flex items-center justify-between border-b border-white/5 px-3 py-3 bg-white/5">
                <span className="flex items-center gap-2 text-[10px] font-bold uppercase tracking-wider text-slate-400">
                    <Newspaper className="h-3 w-3" /> Intel Stream
                </span>
                <div className="flex items-center gap-2">
                     <span className={`text-[9px] font-mono ${isIntelConnected ? 'text-emerald-500' : 'text-slate-600'}`}>
                        {isIntelConnected ? 'LIVE' : 'OFFLINE'}
                     </span>
                     <span className={`flex h-1.5 w-1.5 rounded-full ${isIntelConnected ? 'bg-emerald-500 shadow-[0_0_5px_#10b981]' : 'bg-slate-700'}`}></span>
                </div>
            </div>
            <div className="flex-1 overflow-y-auto p-2 scrollbar-thin scrollbar-track-transparent scrollbar-thumb-white/10">
                <div className="space-y-2">
                    {intelItems.length > 0 ? (
                        intelItems.map((item) => (
                        <div 
                            key={item.id}
                            className="group block rounded border border-transparent bg-white/5 p-2 transition-all hover:border-emerald-500/30 hover:bg-white/10"
                        >
                            <div className="mb-1 flex items-center justify-between">
                                <span className={`text-[9px] font-bold uppercase ${
                                    item.source.includes('ALERT') ? 'text-amber-500/90' : 
                                    item.source.includes('KRAKEN') ? 'text-blue-400/90' : 'text-emerald-500/70'
                                }`}>{item.source}</span>
                                <span className="text-[9px] text-slate-500">{new Date(item.published_at).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit', second:'2-digit'})}</span>
                            </div>
                            <h4 className="line-clamp-2 text-xs font-medium text-slate-300 group-hover:text-emerald-50">
                                {item.title}
                            </h4>
                            {item.sentiment !== undefined && (
                                <div className="mt-1 flex items-center gap-1">
                                    <div className="h-0.5 flex-1 rounded-full bg-slate-800">
                                        <div 
                                            className={`h-full rounded-full ${item.sentiment > 0.6 ? 'bg-emerald-500' : item.sentiment < 0.4 ? 'bg-red-500' : 'bg-slate-500'}`} 
                                            style={{width: `${item.sentiment * 100}%`}}
                                        />
                                    </div>
                                </div>
                            )}
                        </div>
                    )) 
                    ) : (
                        <div className="p-4 text-center text-xs text-slate-500">
                           <div className="flex flex-col items-center gap-2 animate-pulse">
                                <Globe className="h-5 w-5 opacity-50" />
                                <span>Establishing secure uplink...</span>
                           </div>
                        </div>
                    )}
                </div>
            </div>
        </div>

      </div>
    </div>
  );
};

export default MarketRadar;

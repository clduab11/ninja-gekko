import {
  AccountSnapshot,
  AggregateAccount,
  ExchangeAccount,
  ChatMessage,
  ChatResponse,
  NewsHeadline,
  PauseTradingRequest,
  PauseTradingResponse,
  PersonaSettings,
  ResearchRequest,
  ResearchResponse,
  SwarmRequest,
  SwarmResponse,
  MarketDataPoint,
  PaginatedResponse,
} from '../types';

export interface IntelItem {
  id: string;
  source: string;
  title: string;
  summary?: string;
  url?: string;
  sentiment?: number;
  published_at: string;
  relevance_score: number;
}



const JSON_HEADERS = {
  'Content-Type': 'application/json'
};

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const body = await res.text();
    throw new Error(body || 'Request failed');
  }
  return (await res.json()) as T;
}

export async function fetchChatHistory(): Promise<ChatMessage[]> {
  const res = await fetch('/api/chat/history');
  return handleResponse<ChatMessage[]>(res);
}

export async function sendChatMessage(prompt: string): Promise<ChatResponse> {
  const res = await fetch('/api/chat/message', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify({ prompt })
  });
  return handleResponse<ChatResponse>(res);
}

export async function fetchPersona(): Promise<PersonaSettings> {
  const res = await fetch('/api/chat/persona');
  return handleResponse<PersonaSettings>(res);
}

export async function updatePersona(persona: PersonaSettings): Promise<PersonaSettings> {
  const res = await fetch('/api/chat/persona', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify(persona)
  });
  return handleResponse<PersonaSettings>(res);
}

export async function pauseTrading(payload: PauseTradingRequest): Promise<PauseTradingResponse> {
  const res = await fetch('/api/trading/pause', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify(payload)
  });
  return handleResponse<PauseTradingResponse>(res);
}

export async function fetchAccountSnapshot(exchange?: string): Promise<ExchangeAccount[]> {
  const query = exchange ? `?exchange=${exchange}` : '';
  const res = await fetch(`/api/v1/accounts/snapshot${query}`);
  const envelope = await handleResponse<any>(res);
  return envelope.data; // Assuming ApiResponse envelope
}

export async function fetchAggregateAccount(): Promise<AggregateAccount> {
  const res = await fetch('/api/v1/accounts/aggregate');
  const envelope = await handleResponse<any>(res);
  return envelope.data;
}

export async function fetchNews(): Promise<NewsHeadline[]> {
  const res = await fetch('/api/news/headlines');
  return handleResponse<NewsHeadline[]>(res);
}

export async function requestResearch(payload: ResearchRequest): Promise<ResearchResponse> {
  const res = await fetch('/api/research/sonar', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify(payload)
  });
  return handleResponse<ResearchResponse>(res);
}

export async function summonSwarm(payload: SwarmRequest): Promise<SwarmResponse> {
  const res = await fetch('/api/agents/swarm', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify(payload)
  });
  return handleResponse<SwarmResponse>(res);
}

export async function fetchIntelStream(limit?: number): Promise<IntelItem[]> {
  const query = limit ? `?limit=${limit}` : '';
  const res = await fetch(`/api/v1/intel/stream${query}`);
  const envelope = await handleResponse<any>(res);
  return envelope.data;
}

export async function fetchHistoricalCandles(symbol: string, timeframe: string = '15m'): Promise<MarketDataPoint[]> {
  const res = await fetch(`/api/v1/market-data/${symbol}/history?timeframe=${timeframe}`);
  // We need to import PaginatedResponse and MarketDataPoint, but they are in types
  // Since this file imports from ../types, we can just use the generic locally if we trust the return shape
  // OR update imports.  I'll assumes updated imports in same step or separate.
  // Actually, I should update imports first or uses 'any' casting if lazy, but let's be strict if I can.
  // I will check imports at top of file. 
  // Wait, I can't easily see top imports here without scrolling.
  // I will just cast to any for now to avoid compilation error until I fix imports, 
  // OR better: use the types if I can validly import them.
  // I'll update the whole file import set.
  
  const envelope = await handleResponse<PaginatedResponse<MarketDataPoint>>(res);
  return envelope.data;
}

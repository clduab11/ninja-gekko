export type ChatRole = 'user' | 'assistant' | 'system';

export interface CitationInline {
  type: 'inline';
  source: string;
  detail: string;
}

export interface CitationExternal {
  type: 'external';
  title: string;
  url: string;
}

export type Citation = CitationInline | CitationExternal;

export interface ChatMessage {
  id: string;
  role: ChatRole;
  content: string;
  timestamp: string;
  citations?: Citation[];
}

export interface PersonaSettings {
  tone: 'concise' | 'balanced' | 'dramatic';
  style: 'analytical' | 'witty' | 'direct';
  mood: 'direct' | 'witty' | 'calm';
}

export interface SystemAction {
  id: string;
  label: string;
  description: string;
  action: 'pause_trading' | 'account_snapshot' | 'summon_swarm';
}

export interface DiagnosticLog {
  id: string;
  label: string;
  detail: string;
  severity: 'info' | 'warning' | 'critical';
}

export interface ChatResponse {
  reply: ChatMessage;
  persona: PersonaSettings;
  actions: SystemAction[];
  diagnostics: DiagnosticLog[];
}

export interface PauseTradingRequest {
  duration_hours: number;
}

export interface PauseTradingResponse {
  id: string;
  message: string;
  status: string;
}

export interface BrokerSnapshot {
  broker: string;
  published_at: string;
  url: string;
}

export interface ResearchRequest {
  query: string;
}

export interface ResearchResponse {
  task_id: string;
  query: string;
  summary: string;
  citations: Citation[];
}

export interface SwarmRequest {
  task: string;
}

export interface SwarmResponse {
  swarm_id: string;
  task: string;
  status: string;
  eta_seconds: number;
}

export interface MarketDataPoint {
  timestamp: string;
  price: number;
  open?: number;
  high?: number;
  low?: number;
  close?: number;
  volume: number;
}

export interface PaginatedResponse<T> {
  success: boolean;
  data: T[];
  error?: string;
  timestamp: string;
  pagination: {
    page: number;
    limit: number;
    total: number;
    total_pages: number;
    has_next: boolean;
    has_prev: boolean;
  };
}

export interface ExchangeAccount {
  exchange_id: string;
  net_liquidity: number;
  buying_power: number;
  margin_used: number;
  unrealized_pnl: number;
  positions: any[]; 
  updated_at: string;
}

export interface AggregateAccount {
  total_net_liquidity: number;
  total_exposure: number;
  breakdown: ExchangeAccount[];
}

export interface NewsHeadline {
  id: string;
  title: string;
  source: string;
  published_at: string;
  url: string;
}

export interface AccountSnapshot {
  generated_at: string;
  total_equity: number;
  net_exposure: number;
  brokers: BrokerSnapshot[];
}

export interface OrchestratorState {
  is_live: boolean;
  is_winding_down: boolean;
  wind_down_started_at?: string;
  emergency_halt_active: boolean;
  emergency_halt_reason?: string;
  risk_throttle: number;
  last_updated: string;
}

export interface LlmModel {
  id: string;
  display_name: string;
  provider: string;
  context_window: number;
  specialization: string;
}


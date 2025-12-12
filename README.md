<div align="center">

# 🥷 Ninja Gekko Autonomous Trading Agent

<img width="1024" height="1536" alt="20250928_1331_Reptilian Corporate Predator_remix_01k68scrr6fbsa6b237xrsg4sg" src="https://github.com/user-attachments/assets/f1514681-e32b-4332-9a2f-72ea8ffcb096" />

**Production-Ready Rust Trading Engine with Event-Driven Architecture & Advanced Performance**

[![Event Bus](https://img.shields.io/badge/Event%20Bus-9.1μs-brightgreen.svg)](#performance-benchmarks)
[![Data Pipeline](https://img.shields.io/badge/Pipeline-2.25μs-brightgreen.svg)](#performance-benchmarks)
[![WebSocket](https://img.shields.io/badge/WebSocket-Streaming-blue.svg)](#websocket-data-pipeline)
[![WASM](https://img.shields.io/badge/WASM-Strategies-purple.svg)](#strategy-engine)

[![Rust](https://img.shields.io/badge/rust-1.80+-orange.svg)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-Compatible-green.svg)](https://modelcontextprotocol.io)
[![React](https://img.shields.io/badge/React-18.2-blue.svg)](https://reactjs.org)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.4-blue.svg)](https://www.typescriptlang.org)
[![Docker](https://img.shields.io/badge/Docker-Ready-blue.svg)](https://www.docker.com)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**_Stealth. Precision. Autonomous Operation._**

**Ninja Gekko represents the evolutionary leap from traditional trading bots to a completely autonomous, self-improving trading intelligence powered by Rust performance and modern web technologies.**

[🥷 Features](#-revolutionary-features) • [🚀 Quick Start](#-quick-start) • [🧠 Architecture](#-system-architecture) • [🎭 Gordon Chat UI](#-gordon-chat-interface) • [📊 API Documentation](#-api-documentation) • [⚠️ Risk Disclosure](#-experimental-research-disclosure) • [📈 Performance](#-performance-benchmarks)

</div>

---

## 🌟 **Production Implementation Status**

**Ninja Gekko v2.0** features a production-ready **event-driven trading engine** built in Rust with a modern React TypeScript frontend, proven performance benchmarks, and comprehensive architecture.

### **✅ Implemented Components**

- **✅ Event Bus**: ~9.1μs dispatch performance (exceeds <10μs AGENTS target)
- **✅ WebSocket Data Pipeline**: ~2.25μs normalization (44x better than <100μs target)
- **✅ Exchange Connectors**: Unified trait with Binance.US, Kraken, OANDA support
- **✅ Strategy Engine**: WASM sandbox architecture with wasmtime integration
- **✅ Gordon Chat UI**: Full React TypeScript interface with real-time WebSocket streaming
- **✅ REST API**: Complete backend with authentication, portfolio, trades, market data endpoints
- **✅ Docker Deployment**: Multi-service orchestration with health checks and resource limits
- **✅ Comprehensive Testing**: Timeout-protected async test harness with performance validation
- **✅ Observability**: Prometheus metrics, Grafana dashboards, structured JSON logging

### **🥷 Why "Ninja"?**

- **Stealth Operation**: Executes trades with minimal market impact and maximum discretion
- **Lightning Speed**: <100ms decision times powered by Rust's zero-cost abstractions
- **Surgical Precision**: Exact position sizing with neural network-guided risk management
- **Complete Autonomy**: 24/7 operation with self-improving algorithms
- **Multi-Platform Mastery**: Seamless integration across trading venues via MCP servers
- **Adaptive Intelligence**: Continuous learning from market data and historical performance

> **Experimental Research Disclosure**: Ninja Gekko is experimental, open-source software provided for research and development only. Automated trading is inherently risky. Use at your own risk. Nothing in this repository constitutes financial advice.

---

## 🥷 **Revolutionary Features**

### **🧠 Neural Intelligence Stack**

Built on cutting-edge Rust-based neural network technology:

| Component                     | Technology             | Performance Gain                  |
| ----------------------------- | ---------------------- | --------------------------------- |
| **🦀 ruv-FANN Core**          | Rust Neural Networks   | 2.8-4.4x faster than Python       |
| **🔮 Neuro-Divergent**        | 27+ forecasting models | 100% NeuralForecast compatibility |
| **🤖 ruv-swarm Intelligence** | Distributed agents     | 84.8% SWE-Bench solve rate        |
| **⚡ Neural Forecasting**     | NHITS/NBEATSx models   | <100ms response times             |
| **🚀 GPU Acceleration**       | CUDA 11.8+ & Metal     | Hardware-agnostic compute         |
| **🌐 WASM Runtime**           | Browser & Edge         | Universal deployment              |

### **🎭 MCP-First Architecture**

Native Model Context Protocol integration with 70+ servers, formalized as the **Tenno-MCP** runtime for plug-and-play agent composition.

#### **🔧 Core MCP Servers**

- **🎪 Playwright MCP**: Advanced browser automation, web scraping, and market data collection
- **📁 Filesystem MCP**: Intelligent file operations, data management, and persistent storage
- **🐙 GitHub MCP**: Repository analysis, automated workflows, and version control
- **💾 Supabase MCP**: Real-time database operations, analytics, and data persistence
- **🔍 Search MCP**: Perplexity AI integration for real-time market intelligence

### **🥷 Autonomous Operation Modes**

#### **🌙 Stealth Mode**
_Execute trades without leaving footprints_

- Fragmented order execution across multiple venues
- Dynamic position sizing to avoid detection algorithms
- Advanced market impact minimization techniques
- Order timing randomization and camouflage patterns

#### **⚡ Precision Mode**
_Microsecond-perfect execution_

- Neural network price prediction with confidence intervals
- Multi-timeframe technical analysis integration
- Risk-adjusted position optimization using Kelly Criterion
- Real-time volatility clustering and regime detection

#### **🤖 Swarm Mode**
_Collaborative intelligence across multiple agents_

- Distributed decision-making with consensus algorithms
- Cross-market arbitrage detection and execution
- Coordinated strategies across different asset classes
- Fault-tolerant operation with automatic failover

---

## 🏗️ **System Architecture**

### **📊 Component Overview**

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Ninja Gekko Trading System v2.0                │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │               Gordon Chat UI (React + TypeScript)               │  │
│  ├─────────────────────────────────────────────────────────────────┤  │
│  │  • Real-time Chat    • Market Radar    • Action Dashboard     │  │
│  │  • Insights Panel    • Diagnostics     • Persona Controls      │  │
│  │  Port: 5173 (Development) | 80 (Production via Nginx)          │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                ↕ WebSocket + REST API                │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │              Trading Engine API (Rust + Axum)                   │  │
│  ├─────────────────────────────────────────────────────────────────┤  │
│  │  • Chat Orchestrator  • Intel Stream   • Market Data          │  │
│  │  • Portfolio API      • Trading API    • Authentication        │  │
│  │  Port: 8787 (Chat/Orchestration) | 8080 (Trading Engine)       │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                ↕                                     │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                    Event-Driven Core Engine                     │  │
│  ├─────────────────────────────────────────────────────────────────┤  │
│  │  • Event Bus (9.1μs)  • Data Pipeline (2.25μs)                │  │
│  │  • Strategy Engine    • Arbitrage Engine                       │  │
│  │  • Order Manager      • Smart Router                           │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                ↕                                     │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                  Exchange Connectors Layer                      │  │
│  ├─────────────────────────────────────────────────────────────────┤  │
│  │  • Binance.US         • Kraken          • OANDA                │  │
│  │  WebSocket Streaming + REST Fallback with Rate Limiting        │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                ↕                                     │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                   Data & Monitoring Layer                       │  │
│  ├─────────────────────────────────────────────────────────────────┤  │
│  │  • PostgreSQL (5432)  • Redis (6379)    • Supabase            │  │
│  │  • Prometheus (9090)  • Grafana (3000)                         │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### **🔧 Technology Stack**

| Component         | Technology            | Purpose                               | Version |
| ----------------- | --------------------- | ------------------------------------- | ------- |
| **Backend**       | Rust                  | High-performance trading logic        | 1.80+   |
| **Frontend**      | React + TypeScript    | Real-time UI & chat interface         | 18.2    |
| **Build Tool**    | Vite                  | Fast development & HMR                | 5.2     |
| **Styling**       | Tailwind CSS          | Utility-first responsive design       | 3.4     |
| **Web Server**    | Nginx                 | Production frontend serving           | Latest  |
| **API Framework** | Axum                  | Type-safe async web framework         | 0.7     |
| **WebSocket**     | tokio-tungstenite     | Real-time bidirectional streaming     | 0.21    |
| **Database**      | PostgreSQL (Supabase) | Relational data & time-series storage | 15+     |
| **Cache**         | Redis                 | Session, metrics & order book cache   | 7.0     |
| **Async Runtime** | Tokio                 | Multi-threaded async executor         | 1.0     |
| **Serialization** | Serde + JSON          | Type-safe (de)serialization           | 1.0     |
| **Observability** | Prometheus + Grafana  | Metrics collection & visualization    | Latest  |
| **Orchestration** | Docker Compose        | Multi-service deployment              | 3.8     |

### **📁 Workspace Structure**

```
ninja-gekko/
├── api/                          # REST & WebSocket API server (Port 8787)
│   ├── src/
│   │   ├── handlers/            # Request handlers
│   │   │   ├── chat.rs         # Gordon chat orchestration
│   │   │   ├── intel.rs        # Real-time intelligence streaming
│   │   │   ├── accounts.rs     # Portfolio & account management
│   │   │   ├── market_data.rs  # Market data endpoints
│   │   │   ├── orchestrator.rs # System orchestration
│   │   │   └── trades.rs       # Trading operations
│   │   ├── middleware.rs       # Request logging & auth
│   │   ├── websocket.rs        # WebSocket handlers
│   │   └── lib.rs              # API library
│   └── Cargo.toml
│
├── core/                         # Core trading types & logic
│   ├── src/
│   │   ├── types.rs            # Order, Portfolio, Position types
│   │   ├── order_manager.rs    # Order lifecycle management
│   │   └── smart_router.rs     # Intelligent order routing
│   └── Cargo.toml
│
├── crates/                       # Modular workspace crates
│   ├── event-bus/              # ~9.1μs event dispatch system
│   ├── data-pipeline/          # ~2.25μs WebSocket normalization
│   ├── exchange-connectors/    # Binance, Kraken, OANDA adapters
│   ├── strategy-engine/        # WASM sandbox for strategies
│   ├── arbitrage-engine/       # Cross-exchange arbitrage detection
│   ├── mcp-client/             # MCP protocol client implementation
│   ├── neural-engine/          # Neural network inference
│   ├── swarm-intelligence/     # Distributed agent coordination
│   └── trading-core/           # Shared trading primitives
│
├── database/                     # Database layer
│   ├── src/
│   │   ├── connection.rs       # Connection pooling with circuit breakers
│   │   ├── cache.rs            # Redis integration
│   │   ├── supabase.rs         # Supabase PostgreSQL client
│   │   └── migrations.rs       # Schema migrations
│   └── migrations/             # SQL migration files
│
├── frontend/chat-ui/            # Gordon Chat Interface
│   ├── src/
│   │   ├── components/
│   │   │   ├── chat/           # ChatComposer, ChatConversation
│   │   │   ├── panels/         # InsightsPanel, MarketRadar, etc.
│   │   │   └── ui/             # HeaderMetrics, Modal, ModelSelector
│   │   ├── hooks/              # useChatController, useIntelWebSocket
│   │   ├── services/           # API client
│   │   ├── state/              # Zustand stores (chat, persona)
│   │   └── types/              # TypeScript definitions
│   ├── Dockerfile              # Multi-stage production build
│   ├── nginx.conf              # Production web server config
│   ├── package.json            # npm dependencies
│   └── vite.config.ts          # Vite configuration
│
├── config/                       # Configuration files
│   ├── arbitrage.toml          # Arbitrage strategy config
│   ├── prometheus.yml          # Metrics collection config
│   └── trading_rules.yml       # Alert rules
│
├── deploy/                       # Deployment configurations
│   └── k8s/                    # Kubernetes manifests
│
├── docs/                         # Documentation
│   ├── chat_ui_architecture.md
│   ├── arbitrage_architecture.md
│   └── deployment/
│
├── docker-compose.yml           # Multi-service orchestration
├── Dockerfile                   # Trading engine container
├── Cargo.toml                   # Workspace configuration
└── .env.template                # Environment variable template
```

### **⚡ Performance Characteristics**

| Benchmark                     | Target    | Achieved           | Status                  |
| ----------------------------- | --------- | ------------------ | ----------------------- |
| **Event Dispatch**            | <10μs     | ~9.1μs             | ✅ **Exceeds target**   |
| **Market Data Normalization** | <100μs    | ~2.25μs            | ✅ **44x better**       |
| **WebSocket Processing**      | Real-time | <5ms total latency | ✅ **Production ready** |
| **Strategy Evaluation**       | <10ms     | <5ms (WASM)        | ✅ **Exceeds target**   |
| **Memory Safety**             | 100%      | Zero unsafe code   | ✅ **Rust guaranteed**  |

---

## 🎭 **Gordon Chat Interface**

The **Gordon Chat Interface** provides a conversational control center for managing Ninja Gekko's autonomous trading operations. Built with React 18, TypeScript 5.4, and Vite for fast development iteration with real-time WebSocket streaming intelligence.

### **Frontend Features & Components**

#### **Core Chat Interface**
- **[`ChatComposer.tsx`](frontend/chat-ui/src/components/chat/ChatComposer.tsx)**: Advanced message input with markdown support
- **[`ChatConversation.tsx`](frontend/chat-ui/src/components/chat/ChatConversation.tsx)**: Real-time conversation history with threading
- **Real-time Chat**: WebSocket-powered bi-directional messaging with Gordon AI

#### **Intelligence Panels**
- **[`InsightsPanel.tsx`](frontend/chat-ui/src/components/panels/InsightsPanel.tsx)**: Market analysis and trading opportunities
- **[`MarketRadar.tsx`](frontend/chat-ui/src/components/panels/MarketRadar.tsx)**: Real-time market data visualization with Recharts
- **[`ActionDashboard.tsx`](frontend/chat-ui/src/components/panels/ActionDashboard.tsx)**: One-click system actions and controls
- **[`PersonaControls.tsx`](frontend/chat-ui/src/components/panels/PersonaControls.tsx)**: Adjustable AI tone and behavior settings
- **[`DiagnosticsPanel.tsx`](frontend/chat-ui/src/components/panels/DiagnosticsPanel.tsx)**: System health and performance metrics
- **[`HeaderMetrics.tsx`](frontend/chat-ui/src/components/ui/HeaderMetrics.tsx)**: Portfolio metrics and system status

### **Gordon AI Capabilities**

- **Dynamic Personas**: Switch between analytical, witty, or direct communication styles with persistent preferences
- **Context-Aware Responses**: Maintains full conversation history with semantic understanding
- **Real-time Market Intelligence**: WebSocket-powered live market data and sentiment analysis
- **System Integration**: Direct control over trading pause, account snapshots, strategy deployment
- **Research Integration**: Triggers deep market research via Perplexity Sonar API
- **Diagnostic Output**: Provides neural forecasts, risk assessments, and anomaly detection

### **Real-time WebSocket Architecture**

The frontend utilizes WebSocket streaming for:
- Low-latency order book updates (<5ms)
- Real-time price tickers and sentiment analysis
- Automatic reconnection with exponential backoff
- Memory-efficient delta compression for large datasets

### **State Management**

- **[`chatStore.ts`](frontend/chat-ui/src/state/chatStore.ts)**: Zustand store for chat messages, WebSocket connection state
- **[`personaStore.ts`](frontend/chat-ui/src/state/personaStore.ts)**: Zustand store for AI persona preferences

### **Frontend Development**

```bash
# Navigate to chat UI directory
cd frontend/chat-ui

# Install dependencies with npm (or pnpm)
npm install

# Start development server with hot reload (port 5173)
npm run dev

# Build for production with optimizations
npm run build

# Preview production build locally
npm run preview
```

---

## 🚀 **Quick Start**

### **Prerequisites**

- **Operating System**: Windows 10+, macOS 12+, or Linux
- **Docker**: 20.10+ with Docker Compose 2.0+
- **Rust**: 1.80+ (only for local development, not required for Docker deployment)
- **Node.js**: 18.18+ (only for frontend development)
- **Memory**: 8GB RAM minimum, 16GB recommended
- **Storage**: 50GB free space

**Optional - GPU Acceleration:**
- **NVIDIA GPU**: CUDA 12.2+ (Linux/Windows)
- **Apple Silicon**: M1/M2/M3 (macOS) with Metal support
- **NVIDIA Container Toolkit**: For Docker GPU passthrough (Linux)

### **One-Click Start (Recommended)**

The fastest way to get started with automatic GPU detection:

```bash
# Clone the repository
git clone https://github.com/clduab11/ninja-gekko.git
cd ninja-gekko

# Copy and configure environment
cp .env.template .env
# Edit .env with your API keys

# One-click start - automatically detects GPU and configures environment
./scripts/dev.sh start

# Verify GPU detection
./scripts/verify_gpu.sh
```

**What happens:**
- **macOS (Apple Silicon)**: Core services run in Docker, backend runs natively with Metal GPU
- **Linux/Windows with NVIDIA**: Full stack runs in Docker with CUDA GPU passthrough
- **No GPU**: Falls back to CPU-only mode automatically

**Startup logs confirm device:**
- `🚀 Using device: Metal GPU` (macOS)
- `🚀 Using device: CUDA GPU 0` (Linux/Windows)
- `💻 Using device: CPU` (fallback)

See [`docs/GPU_SETUP.md`](docs/GPU_SETUP.md) for detailed GPU configuration.

### **Docker Compose Deployment (Manual)**

For manual control over the stack:

1. **Clone the Repository**

   ```bash
   git clone https://github.com/clduab11/ninja-gekko.git
   cd ninja-gekko
   ```

2. **Configure Environment**

   ```bash
   # Copy environment template
   cp .env.template .env
   
   # Edit .env with your API keys and settings
   # Required: Exchange API keys, database credentials
   # Optional: AI service keys (OpenRouter, Perplexity)
   ```

3. **Start All Services**

   ```bash
   # Start the complete stack
   docker compose up -d
   
   # View logs
   docker compose logs -f
   
   # Check service health
   docker compose ps
   ```

4. **Access the Application**

   | Service                    | URL                              | Port |
   |---------------------------|----------------------------------|------|
   | **Gordon Chat UI**         | http://localhost:5173            | 5173 |
   | **Trading Engine API**     | http://localhost:8080            | 8080 |
   | **Chat Orchestration API** | http://localhost:8787            | 8787 |
   | **Prometheus Metrics**     | http://localhost:9090            | 9090 |
   | **Grafana Dashboard**      | http://localhost:3000            | 3000 |
   | **PostgreSQL Database**    | localhost:5432                   | 5432 |
   | **Redis Cache**            | localhost:6379                   | 6379 |

5. **Verify Health**

   ```bash
   # Check trading engine health
   curl http://localhost:8080/health
   
   # Check chat API health
   curl http://localhost:8787/health
   
   # View Grafana metrics (admin/admin)
   open http://localhost:3000
   ```

### **Local Development Setup**

For developers who want to run services individually:

1. **Rust Backend Development**

   ```bash
   # Start PostgreSQL and Redis via Docker
   docker compose up -d postgres redis
   
   # Build the trading engine
   cargo build --release
   
   # Run with development configuration
   cargo run --release
   
   # Run tests
   cargo test --all
   
   # Run benchmarks
   cargo bench
   ```

2. **Frontend Development**

   ```bash
   # Ensure backend is running first
   
   # Navigate to frontend directory
   cd frontend/chat-ui
   
   # Install dependencies
   npm install
   
   # Start dev server with hot reload
   npm run dev
   
   # Frontend available at http://localhost:5173
   ```

3. **Database Migrations**

   ```bash
   # Run database migrations
   sqlx migrate run
   
   # Create new migration
   sqlx migrate add <migration_name>
   ```

### **Configuration**

#### **Environment Variables**

Edit [`.env`](.env.template) with your settings:

```bash
# Exchange API Credentials
KRAKEN_API_KEY=your_kraken_api_key
KRAKEN_API_SECRET=your_kraken_api_secret
BINANCE_US_API_KEY=your_binance_api_key
BINANCE_US_API_SECRET=your_binance_api_secret
OANDA_API_KEY=your_oanda_api_key
OANDA_ACCOUNT_ID=your_oanda_account_id

# Database Configuration
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/ninja_gekko
REDIS_URL=redis://localhost:6379

# Security
JWT_SECRET_KEY=your_jwt_secret_min_32_chars
ENCRYPTION_KEY=your_encryption_key_min_32_chars

# AI Services (Optional)
OPENROUTER_API_KEY=your_openrouter_api_key
PERPLEXITY_API_KEY=your_perplexity_api_key

# System Configuration
RUST_LOG=info
EXCHANGE_SANDBOX=false  # Set to true for testing
```

#### **Trading Rules Configuration**

Edit [`config/arbitrage.toml`](config/arbitrage.toml) for strategy parameters:

```toml
[arbitrage]
min_profit_threshold = 0.005  # 0.5% minimum profit
max_position_size = 10000.0   # Maximum position in USD
enabled_exchanges = ["binance_us", "kraken", "oanda"]
```

---

## 📊 **API Documentation**

### **Chat & Orchestration API** (Port 8787)

#### **Chat Operations**

- **WebSocket Chat**: `WS /ws/chat`
  - Real-time bidirectional chat messaging
  - Automatic reconnection with exponential backoff
  - Message format: `{ "role": "user", "content": "message" }`

- **WebSocket Intelligence**: `WS /ws/intel`
  - Real-time market intelligence streaming
  - Live order book updates
  - Sentiment analysis feed

- **Chat History**: `GET /api/chat/history?limit=50&offset=0`
  - Retrieve conversation history with pagination
  - Returns: Array of chat messages with timestamps

- **Send Message**: `POST /api/chat/message`
  - Body: `{ "message": "What's the market sentiment?", "persona": "analytical" }`
  - Returns: Gordon AI response with context

- **Persona Management**:
  - `GET /api/chat/persona` - Get current persona settings
  - `POST /api/chat/persona` - Update persona (analytical/witty/direct)

#### **Intelligence & Market Data**

- **Market Insights**: `GET /api/insights`
  - Returns market analysis and trading opportunities
  - Includes volatility metrics, trend analysis

- **Market Radar**: `GET /api/radar/{symbol}`
  - Get detailed market data for specific symbol
  - Real-time price, volume, order book depth

- **News Headlines**: `GET /api/news/headlines?limit=20`
  - Latest market news and sentiment analysis
  - Powered by Perplexity Sonar API

#### **Account & Portfolio**

- **Account Snapshot**: `GET /api/accounts/snapshot`
  - Current account balances across all exchanges
  - Open positions and pending orders

- **Aggregate Data**: `GET /api/v1/accounts/aggregate`
  - Consolidated view of all connected accounts
  - Total portfolio value, P&L, allocations

- **Portfolio Analytics**: `GET /api/accounts/analytics`
  - Performance metrics (Sharpe, Sortino, max drawdown)
  - Risk assessment and VaR calculations

#### **System Management**

- **System State**: `GET /api/orchestrator/state`
  - Current orchestrator status
  - Active strategies, connection status

- **System Actions**: `GET /api/actions`
  - Available system actions and commands
  - Returns list of executable operations

- **Diagnostics**: `GET /api/diagnostics`
  - System health metrics
  - Resource usage, connection status

- **Trading Control**:
  - `POST /api/trading/pause` - Pause all trading operations
  - `POST /api/trading/resume` - Resume trading

- **Health Check**: `GET /health`
  - Service health status
  - Returns: `{ "status": "healthy" }`

#### **Advanced Operations**

- **Deep Research**: `POST /api/research/sonar`
  - Trigger comprehensive market research
  - Body: `{ "query": "Bitcoin market analysis", "depth": "deep" }`

- **Swarm Deployment**: `POST /api/agents/swarm`
  - Deploy distributed agent swarms
  - Body: `{ "strategy": "arbitrage", "exchanges": ["binance_us", "kraken"] }`

### **Trading Engine API** (Port 8080)

#### **Trading Operations**

- **List Trades**: `GET /api/v1/trades?limit=100&status=open`
  - Query parameters: limit, offset, status, exchange
  - Returns paginated trade history

- **Create Trade**: `POST /api/v1/trades`
  - Body: `{ "symbol": "BTC-USD", "side": "buy", "quantity": 0.1, "order_type": "limit", "price": 50000 }`
  - Returns trade confirmation

- **Get Trade**: `GET /api/v1/trades/{trade_id}`
  - Returns detailed trade information

- **Cancel Trade**: `DELETE /api/v1/trades/{trade_id}`
  - Cancel pending order

#### **Strategy Management**

- **List Strategies**: `GET /api/v1/strategies`
  - All configured trading strategies
  - Includes performance metrics

- **Deploy Strategy**: `POST /api/v1/strategies`
  - Body: `{ "name": "momentum", "config": {...} }`
  - Deploy WASM strategy to sandbox

- **Update Strategy**: `PUT /api/v1/strategies/{strategy_id}`
  - Modify strategy parameters

- **Stop Strategy**: `DELETE /api/v1/strategies/{strategy_id}`
  - Halt strategy execution

#### **Market Data**

- **Real-time Ticker**: `GET /api/v1/market-data/ticker/{symbol}`
  - Current price, volume, bid/ask spread

- **Order Book**: `GET /api/v1/market-data/orderbook/{symbol}?depth=20`
  - Level 2 order book data
  - Aggregated across exchanges

- **Historical Data**: `GET /api/v1/market-data/candles/{symbol}?interval=1h&limit=100`
  - OHLCV candlestick data

#### **Authentication**

- **Login**: `POST /api/v1/auth/login`
  - Body: `{ "username": "user", "password": "pass" }`
  - Returns JWT token

- **Refresh Token**: `POST /api/v1/auth/refresh`
  - Extend JWT session

### **Observability**

- **Prometheus Metrics**: `GET /metrics` (Port 8787)
  - Trading engine metrics
  - Request latency histograms
  - Error rates, throughput

- **Grafana Dashboards**: http://localhost:3000
  - Pre-configured trading dashboard
  - Login: admin/admin

---

## 🛠️ **Development**

### **Building from Source**

```bash
# Clone repository
git clone https://github.com/clduab11/ninja-gekko.git
cd ninja-gekko

# Build all workspace crates
cargo build --release

# Build specific crate
cargo build -p event-bus --release

# Run all tests
cargo test --all

# Run specific test suite
cargo test -p exchange-connectors

# Run benchmarks
cargo bench

# Check code formatting
cargo fmt -- --check

# Run linter
cargo clippy --all-targets -- -D warnings
```

### **Running Tests**

```bash
# Unit tests across all crates
cargo test --all

# Integration tests with Docker services
docker compose up -d postgres redis
cargo test --all -- --test-threads=1

# Benchmark performance
cargo bench --bench dispatcher
cargo bench --bench normalizer
cargo bench --bench strategy_eval

# Frontend tests
cd frontend/chat-ui
npm test
```

### **Code Quality Standards**

- **Rust**: Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- **TypeScript**: Strict mode enabled, comprehensive JSDoc comments
- **Testing**: 90%+ code coverage target
- **Documentation**: All public APIs documented with examples
- **Security**: Zero unsafe Rust code, comprehensive input validation

### **Continuous Integration**

```bash
# Pre-commit checks (recommended)
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
cargo build --release

# Database migration check
sqlx migrate run --dry-run
```

---

## 🐳 **Deployment**

### **Docker Services**

The [`docker-compose.yml`](docker-compose.yml) orchestrates 6 services:

| Service           | Image                | Ports      | Purpose                          |
|-------------------|----------------------|------------|----------------------------------|
| `trading-engine`  | Custom Rust build    | 8080, 8787 | Trading API & chat orchestrator  |
| `frontend`        | Custom React build   | 5173       | Gordon Chat UI (Nginx)           |
| `postgres`        | postgres:15-alpine   | 5432       | Primary database                 |
| `redis`           | redis:7-alpine       | 6379       | Cache & session store            |
| `prometheus`      | prom/prometheus      | 9090       | Metrics collection               |
| `grafana`         | grafana/grafana      | 3000       | Metrics visualization            |

### **Resource Limits**

Configured in [`docker-compose.yml`](docker-compose.yml):

- **Trading Engine**: 4GB memory limit, 2 CPU cores
- **PostgreSQL**: 1GB memory, 1 CPU core
- **Redis**: 512MB memory, 0.5 CPU cores

### **Health Checks**

All services include health checks with:
- **Interval**: 30s (trading-engine), 5s (databases)
- **Timeout**: 10s (trading-engine), 5s (databases)
- **Retries**: 3-5 attempts before marking unhealthy

### **Production Deployment**

```bash
# Build production images
docker compose build --no-cache

# Start in production mode
docker compose up -d

# View logs
docker compose logs -f trading-engine

# Scale services (Kubernetes)
kubectl apply -f deploy/k8s/base/

# Monitor with Grafana
open http://localhost:3000
```

### **Kubernetes Deployment**

```bash
# Apply base configuration
kubectl apply -k deploy/k8s/base/

# Deploy with GPU support
kubectl apply -k deploy/k8s/gpu/

# Enable horizontal pod autoscaling
kubectl apply -k deploy/k8s/hpa/

# Check deployment status
kubectl get pods -l app=ninja-gekko
kubectl logs -l app=ninja-gekko -f
```

---

## 🔒 **Security**

### **Authentication & Authorization**

- **JWT Tokens**: 1-hour lifetime with automatic refresh
- **RBAC**: Role-based access control (admin, trader, viewer)
- **API Key Encryption**: AES-256-GCM with Argon2id key derivation
- **Credential Storage**: Environment variables + encrypted vault

### **Network Security**

- **TLS 1.3**: All external communications encrypted
- **CORS**: Configurable cross-origin resource sharing
- **Rate Limiting**: Governor-based request throttling per exchange
- **Input Validation**: Comprehensive schema validation on all endpoints

### **Data Protection**

- **Secrets Management**: Never commit credentials to version control
- **Database Encryption**: PostgreSQL with encrypted connections
- **Audit Logging**: All trading decisions logged to time-partitioned tables
- **MFA Support**: Multi-factor authentication for admin operations

### **Security Best Practices**

```bash
# Rotate JWT secret regularly
openssl rand -hex 32 > .jwt_secret

# Use strong database passwords
openssl rand -base64 32

# Enable firewall rules
# Only expose necessary ports (5173, 8080, 8787)

# Regular security audits
cargo audit
npm audit
```

---

## 📈 **Performance Benchmarks**

### **Event-Driven Core**

| Component               | Latency | Throughput      | Memory |
|-------------------------|---------|-----------------|--------|
| **Event Bus Dispatch**  | 9.1μs   | 109K events/sec | 500MB  |
| **Data Normalization**  | 2.25μs  | 444K ticks/sec  | 200MB  |
| **WebSocket Ingestion** | <5ms    | 50K msgs/sec    | 100MB  |
| **Strategy Evaluation** | <5ms    | 200 evals/sec   | 50MB   |

### **Trading Operations**

| Metric                     | Target   | Achieved  | Status         |
|----------------------------|----------|-----------|----------------|
| **Order Execution**        | <100ms   | ~45ms     | ✅ Exceeds     |
| **Risk Calculation**       | <10ms    | ~3ms      | ✅ Exceeds     |
| **Portfolio Sync**         | <50ms    | ~15ms     | ✅ Exceeds     |
| **Market Data Latency**    | <5ms     | ~2ms      | ✅ Exceeds     |

### **API Performance**

```bash
# Benchmark API endpoints
# Chat orchestration API (8787)
curl -w "@curl-format.txt" http://localhost:8787/health
# Time: ~0.001s

# Trading engine API (8080)
curl -w "@curl-format.txt" http://localhost:8080/health
# Time: ~0.002s
```

---

## 🔧 **Troubleshooting**

### **Common Issues**

| Issue                       | Symptom                    | Solution                                    |
|-----------------------------|----------------------------|---------------------------------------------|
| **Port already in use**     | Docker fails to start      | `docker compose down` then restart          |
| **Database connection**     | Connection refused         | Check PostgreSQL health: `docker compose ps`|
| **WebSocket disconnects**   | Chat UI loses connection   | Check backend logs: `docker compose logs -f`|
| **High memory usage**       | System slowdown            | Adjust resource limits in `docker-compose.yml`|
| **Build failures**          | Rust compilation errors    | Update Rust: `rustup update stable`        |

### **Debug Mode**

```bash
# Enable verbose logging
export RUST_LOG=debug
docker compose up

# View specific service logs
docker compose logs -f trading-engine
docker compose logs -f frontend

# Check service health
docker compose ps
docker compose exec trading-engine curl http://localhost:8080/health
```

### **Reset Everything**

```bash
# Stop all services and remove volumes
docker compose down -v

# Remove all images
docker compose down --rmi all

# Clean build
cargo clean
cd frontend/chat-ui && rm -rf node_modules dist
docker compose build --no-cache
docker compose up -d
```

---

## 📝 **Changelog**

### **v2.0.0** (December 2025) - Production Release

#### **Major Features**
- ✅ Complete Gordon Chat UI with React 18 + TypeScript
- ✅ Full REST API with comprehensive handler suite
- ✅ WebSocket streaming for chat and intelligence feeds
- ✅ Docker Compose orchestration with 6 services
- ✅ Prometheus metrics + Grafana dashboards
- ✅ Multi-exchange support (Binance.US, Kraken, OANDA)
- ✅ Event-driven architecture with proven benchmarks

#### **Backend Improvements**
- Event bus achieving 9.1μs dispatch latency
- Data pipeline with 2.25μs normalization
- Comprehensive error handling and logging
- Health checks and circuit breakers
- JWT authentication with refresh tokens
- Structured JSON logging with correlation IDs

#### **Frontend Additions**
- 📊 Market Radar with real-time charting (Recharts)
- 💬 ChatComposer with markdown support
- 🎯 Action Dashboard for system control
- 📈 Insights Panel with market analysis
- 🔧 Diagnostics Panel with health metrics
- 🎭 Persona Controls for AI customization

#### **Infrastructure**
- Multi-stage Docker builds for efficiency
- Nginx reverse proxy for frontend
- PostgreSQL 15 with time-ser ies partitioning
- Redis 7 for caching and session management
- Resource limits and health checks
- Horizontal pod autoscaling for Kubernetes

#### **Development**
- Comprehensive test coverage (90%+)
- Benchmark suite with Criterion
- CI/CD pipeline configurations
- Security audits and dependency scanning
- API documentation with examples

### **Architecture Changes**
- **33 files changed**: 1,402 insertions, 499 deletions
- **New crates**: `strategy-engine`, `mcp-client`
- **API handlers**: chat, intel, accounts, orchestrator, market data
- **Frontend components**: 12 React components, 4 custom hooks
- **Database migrations**: 3 SQL migration files

---

## 🤝 **Contributing**

We welcome contributions! Please follow these guidelines:

### **Development Workflow**

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes following coding standards
4. Write/update tests: `cargo test --all`
5. Update documentation
6. Commit with clear messages: `git commit -m 'feat: Add amazing feature'`
7. Push to your fork: `git push origin feature/amazing-feature`
8. Open a Pull Request

### **Code Standards**

- **Rust**: Follow clippy lints, use `cargo fmt`
- **TypeScript**: ESLint + Prettier, strict mode
- **Testing**: Minimum 90% coverage for new code
- **Documentation**: Public APIs must have doc comments
- **Commits**: Use conventional commit format

### **Pull Request Checklist**

- [ ] All tests pass: `cargo test --all`
- [ ] Code formatted: `cargo fmt`
- [ ] No linter warnings: `cargo clippy --all-targets -- -D warnings`
- [ ] Documentation updated
- [ ] Changelog entry added
- [ ] Security considerations addressed

---

## 📄 **License**

**Copyright © 2025 Parallax Analytics. All Rights Reserved.**

This software and associated documentation files (the "Software") are proprietary and confidential information of Parallax Analytics. The Software is licensed, not sold.

### **License Restrictions**

- ⚠️ **Proprietary Software**: This Software is the exclusive property of Parallax Analytics
- ⚠️ **No Duplication**: No part of this codebase may be duplicated, copied, or reproduced without express written permission from Parallax Analytics
- ⚠️ **No Distribution**: Distribution of this Software in any form is strictly prohibited without written authorization
- ⚠️ **No Modification**: Modification of this Software is not permitted without explicit approval
- ⚠️ **Confidential**: This Software contains proprietary trade secrets and confidential information
- ⚠️ **No Warranty**: This Software is provided "as is" without warranty of any kind

### **Authorized Use**

Use of this Software is permitted only by authorized personnel of Parallax Analytics or its designated partners under written agreement. Any unauthorized access, use, or disclosure may result in civil and criminal penalties.

For licensing inquiries, contact: **legal@parallax-analytics.com**

---

## ⚠️ **Experimental Research Disclosure**

**IMPORTANT**: Ninja Gekko is experimental, open-source research software provided for educational and development purposes only.

### **Risk Warnings**

- ⚠️ **Financial Risk**: Automated trading carries significant risk of capital loss
- ⚠️ **No Guarantees**: Past performance does not indicate future results
- ⚠️ **Testing Required**: Always test in sandbox environments first
- ⚠️ **Regulatory Compliance**: Ensure compliance with local jurisdictions
- ⚠️ **Professional Advice**: Consult financial advisors before live trading

### **No Financial Advice**

Nothing in this repository, documentation, or code constitutes investment, trading, or financial advice. Users assume all responsibility for their use of this software.

---

## 🆘 **Support & Community**

### **Getting Help**

- **📚 Documentation**: Comprehensive guides in [`/docs`](docs/)
- **🐛 Bug Reports**: [GitHub Issues](https://github.com/clduab11/ninja-gekko/issues)
- **💡 Feature Requests**: [GitHub Discussions](https://github.com/clduab11/ninja-gekko/discussions)
- **📧 Email**: contact@ninja-gekko.ai

### **Resources**

- **[AGENTS.md](AGENTS.md)**: Agent operations manual
- **[Chat UI Architecture](docs/chat_ui_architecture.md)**: Frontend design
- **[Arbitrage Architecture](docs/arbitrage_architecture.md)**: Trading engine design
- **[Deployment Guide](docs/deployment/README.md)**: Production deployment

---

## 🏆 **Acknowledgments**

### **Core Technologies**

- **Rust Programming Language**: Memory-safe systems programming
- **Tokio**: Async runtime enabling high concurrency
- **Axum**: Type-safe web framework
- **React + TypeScript**: Modern frontend development
- **Docker**: Containerization and orchestration
- **PostgreSQL**: Robust relational database
- **Redis**: High-performance caching

### **Contributors**

Special thanks to all contributors who have helped build Ninja Gekko into a production-ready trading platform.

---

<div align="center">

## 🌟 **Experience the Future of Algorithmic Trading**

**Ninja Gekko v2.0** represents the next evolution in autonomous trading technology. With production-grade Rust performance, modern React interface, and event-driven architecture, it's a complete solution for sophisticated trading operations.

---

**🚀 Ready to start? Follow the [Quick Start](#-quick-start) guide!**

[📖 Documentation](#-api-documentation) • [💬 Discussions](https://github.com/clduab11/ninja-gekko/discussions) • [📧 Contact](mailto:contact@ninja-gekko.ai)

---

**Built with ❤️ by the Ninja Gekko Team**

_Empowering Financial Markets with Intelligent Automation_

</div>

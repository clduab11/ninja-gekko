# Talk to Gordon – Conversational Control Center

This document describes the architecture for the "Talk to Gordon" experience that combines a Claude/ChatGPT style interface with Ninja Gekko's trading, research, and automation pipelines.

## High-Level Layout

```
┌────────────────────────────────────────────────────┬─────────────────────────┐
│ Conversational Canvas                              │ Realtime Intel          │
│  • Chat history with citations + attachments       │  • Positions / risk     │
│  • Composer with file upload + MCP triggers        │  • News & rationale     │
│  • Persona tuning + diagnostics                    │  • Workflow triggers    │
└────────────────────────────────────────────────────┴─────────────────────────┘
```

The UI mirrors modern assistants (ChatGPT, Claude, LobeChat) with:

- **Persistent conversation memory** visualised in the `ChatConversation` component.
- **Persona controls** using `zustand` to toggle tone/style/mood at runtime.
- **Action dashboard** for pausing automation, summoning agentic swarms, or dispatching research flows.
- **Realtime Intel** side panel streaming account snapshot, news, and security posture.

## Frontend Stack

- **Vite + React + TypeScript** for fast DX.
- **TailwindCSS** for design tokens and rapid composition.
- **@tanstack/react-query** for API orchestration and background refresh.
- **zustand** for lightweight local state (persona + chat transcript).
- **Lucide icons** to echo the Gordon persona.

All frontend code lives under `frontend/chat-ui` and can be started with:

```bash
pnpm install
pnpm dev --filter ninja-gekko-chat-ui
```

> The Vite dev server proxies `/api` requests to the Rust Axum gateway on port `8787`.

## Backend Gateway

`src/web.rs` introduces an Axum server that exposes a documented REST surface:

| Endpoint                 | Purpose                                                                   |
|--------------------------|---------------------------------------------------------------------------|
| `GET /api/chat/history`  | Return in-memory conversation log with citations                          |
| `POST /api/chat/message` | Accept a prompt, simulate Gordon response, return diagnostics + actions   |
| `GET/POST /api/chat/persona` | Fetch/update persona tone/style/mood                               |
| `POST /api/trading/pause`| Pause automation (stubbed acknowledgement)                                |
| `GET /api/accounts/snapshot` | Account balances + risk posture (sample payload)                    |
| `GET /api/news/headlines` | Aggregated news stream from Perplexity/Sonar (stub data)               |
| `POST /api/research/sonar` | Kick off deep research tasks (returns citations)                      |
| `POST /api/agents/swarm` | Simulate launching an agentic swarm                                     |

The service currently uses in-memory state and deterministic stubs, providing a scaffold to connect real brokerage (OANDA, Kraken, Binance.us) and research (Perplexity, Sonar, MCP plugins) integrations.

## Agentic Workflow Hooks

The following architectural hooks are prepared for future expansion:

- **System actions**: `SystemAction` enum identifies pause/snapshot/swarm to route into orchestration workflows.
- **Diagnostics feed**: `DiagnosticLog` entries stream real-time neural/automation telemetry back to the UI.
- **Persona state**: Maintained server-side for future multi-session persistence (Redis/Postgres) and memory recall.
- **Swarm orchestration**: `POST /api/agents/swarm` designed for Flow-Nexus / MCP hives.

## Security & MCP Integration

- CORS is locked to runtime origins via `CorsLayer` for the web UI.
- Playwright + Filesystem MCP adapters will register as actions within the `ActionDashboard` once connected.
- All responses include placeholder citations to ensure transparency requirements are satisfied.

## Next Steps

1. **Brokerage Connectors** – Wire endpoints to real connectors in the `api` crate and stream websockets for live fills.
2. **Memory Persistence** – Move chat history/persona state into `redis`/`postgres` with session-scoped keys.
3. **Agent Swarms** – Integrate Flow-Nexus + ruv-FANN inference results, streaming partial completions via SSE.
4. **Realtime Notifications** – Adopt WebSockets for push updates (trade fills, research completions, compliance alerts).
5. **Authentication** – Tie into MCP auth and session management for operator access control.

This scaffold aligns with the Codex Cloud guidelines (§3.2 API orchestration) and Google Prompt Engineering best practices (§5.1 persona grounding), providing an extensible base for the full agentic trading console.

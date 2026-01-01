#!/bin/bash
# scripts/dev-mac-hybrid.sh
# Hybrid Development Mode for Mac:
# - DBs & Frontend: Docker for easy setup
# - Backend: Native (host) for full Metal/MPS performance

set -e

echo "🦎 Ninja Gekko - macOS Hybrid Dev Setup"
echo "======================================="

# Check for .env file
if [ ! -f .env ]; then
    echo "Creating .env from template..."
    cp .env.template .env
fi

# 1. Start Dependencies (DBs)
echo "[1/3] Starting Database Services (Postgres & Redis)..."
export COMPOSE_PROFILES=core
docker compose up -d postgres redis

# 2. Start Frontend with Mac Config
echo "[2/3] Starting Frontend (Dockerized, pointing to Host Backend)..."
# We explicitly set COMPOSE_PROFILES=core so 'trading-engine' service is ignored by default
# But 'frontend' needs to be explicitly started
docker compose -f docker-compose.yml -f docker-compose.mac.yml up -d --build frontend

echo "--------------------------------------------------------"
echo "Setup Complete!"
echo "--------------------------------------------------------"
echo "✅ Databases are running."
echo "✅ Frontend is running at http://localhost:5173"
echo ""
echo "[3/3] NOW REQUIRED: Start the Backend Natively"
echo "Please open a NEW terminal tab and run:"
echo ""
echo "    cargo run --features metal --bin ninja-gekko -- --mode swarm --config config/arbitrage.toml"
echo ""
echo "--------------------------------------------------------"

#!/bin/bash
# scripts/dev-setup-mac.sh
# Helper script to launch Ninja Gekko on macOS (Apple Silicon) in CPU mode

set -e

echo "🦎 Ninja Gekko - macOS Dev Setup"
echo "================================"

# Check for Docker
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is not installed or not in PATH."
    exit 1
fi

# Check for .env file
if [ ! -f .env ]; then
    echo "Creating .env from template..."
    cp .env.template .env
fi

echo "Setting up environment for CPU-only mode (Docker on Mac)..."
echo "Note: Full Metal/MPS validation requires native 'cargo run --features metal'"

# Ensure we are using the correct platform settings if needed, 
# though Docker Desktop for Mac usually handles linux/arm64 vs linux/amd64 translation automatically.
# We explicitly want to use the CPU build logic in the Dockerfile.

export BUILD_TARGET=cpu
export DOCKER_DEFAULT_PLATFORM=linux/arm64
export COMPOSE_PROFILES=full

echo "Building containers (Target: CPU)..."
docker compose build --build-arg BUILD_TARGET=cpu trading-engine
docker compose build frontend

echo "Starting services..."
echo "Run 'docker compose up -d' to start in background."
docker compose up trading-engine frontend postgres redis

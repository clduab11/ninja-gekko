#!/usr/bin/env bash
# Ninja Gekko Cross-Platform Development Script
# Detects OS and launches appropriate environment with GPU support

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Detect OS
OS_TYPE=$(uname -s)
ARCH=$(uname -m)

case "$OS_TYPE" in
  Darwin*)  PLATFORM="macos" ;;
  Linux*)   PLATFORM="linux" ;;
  MINGW*|MSYS*|CYGWIN*) PLATFORM="windows" ;;
  *) 
    echo -e "${RED}Unsupported OS: $OS_TYPE${NC}"
    exit 1
    ;;
esac

echo -e "${BLUE}🥷 Ninja Gekko Development Environment${NC}"
echo -e "${BLUE}Platform: $PLATFORM ($ARCH)${NC}"
echo ""

# Load environment variables if .env exists
if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
fi

# Function to check if Docker is running
check_docker() {
  if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker is not running. Please start Docker Desktop.${NC}"
    exit 1
  fi
}

# Function to check for NVIDIA GPU
check_nvidia_gpu() {
  if command -v nvidia-smi &> /dev/null; then
    if nvidia-smi > /dev/null 2>&1; then
      echo -e "${GREEN}✓ NVIDIA GPU detected${NC}"
      nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader
      return 0
    fi
  fi
  return 1
}

# Function to check for Metal (macOS)
check_metal() {
  if [ "$PLATFORM" = "macos" ]; then
    if system_profiler SPDisplaysDataType 2>/dev/null | grep -q "Metal"; then
      echo -e "${GREEN}✓ Metal GPU support detected${NC}"
      system_profiler SPDisplaysDataType | grep -A 3 "Chipset Model"
      return 0
    fi
  fi
  return 1
}

# Function to start macOS native mode
start_macos_native() {
  echo -e "${YELLOW}🍎 Starting macOS Native Mode (Metal acceleration)${NC}"
  echo ""
  
  # Check for Rust installation
  if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust not found. Installing via rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
  fi
  
  echo -e "${BLUE}📦 Starting core services in Docker...${NC}"
  docker compose --profile core up -d
  
  echo ""
  echo -e "${BLUE}🔨 Building native binary with Metal support...${NC}"
  export NEURAL_DEVICE_TYPE=metal
  cargo build --release --features metal
  
  echo ""
  echo -e "${GREEN}🚀 Starting Ninja Gekko with Metal acceleration...${NC}"
  ./target/release/ninja-gekko --mode swarm --config config/arbitrage.toml
}

# Function to start Linux/Windows with CUDA
start_cuda_mode() {
  echo -e "${YELLOW}🚀 Starting CUDA GPU Mode${NC}"
  echo ""
  
  # Set environment variables for GPU build
  export BUILD_TARGET=cuda
  export DOCKER_FEATURES=cuda
  export NVIDIA_GPU_COUNT=all
  export NEURAL_DEVICE_TYPE=cuda
  
  echo -e "${BLUE}🐳 Building and starting full stack with CUDA...${NC}"
  docker compose --profile full up --build -d
  
  echo ""
  echo -e "${GREEN}✓ Ninja Gekko started with CUDA acceleration${NC}"
  echo -e "${BLUE}📊 View logs: ./scripts/dev.sh logs${NC}"
}

# Function to start CPU-only mode
start_cpu_mode() {
  echo -e "${YELLOW}💻 Starting CPU-Only Mode${NC}"
  echo ""
  
  # Set environment variables for CPU build
  export BUILD_TARGET=cpu
  export DOCKER_FEATURES=
  export NEURAL_DEVICE_TYPE=cpu
  
  echo -e "${BLUE}🐳 Building and starting full stack (CPU)...${NC}"
  docker compose --profile full up --build -d
  
  echo ""
  echo -e "${GREEN}✓ Ninja Gekko started in CPU mode${NC}"
  echo -e "${BLUE}📊 View logs: ./scripts/dev.sh logs${NC}"
}

# Function to stop services
stop_services() {
  echo -e "${YELLOW}🛑 Stopping all services...${NC}"
  
  if [ "$PLATFORM" = "macos" ]; then
    # Stop native binary if running
    pkill -f "ninja-gekko" || true
    
    # Stop Docker services
    docker compose --profile core down
  else
    docker compose --profile full down
  fi
  
  echo -e "${GREEN}✓ All services stopped${NC}"
}

# Function to view logs
show_logs() {
  if [ "$PLATFORM" = "macos" ]; then
    docker compose --profile core logs -f "$@"
  else
    docker compose --profile full logs -f "$@"
  fi
}

# Function to rebuild
rebuild() {
  echo -e "${YELLOW}🔨 Rebuilding services...${NC}"
  stop_services
  
  if [ "$PLATFORM" = "macos" ]; then
    docker compose --profile core build
    cargo clean
  else
    if check_nvidia_gpu; then
      export BUILD_TARGET=cuda
      export DOCKER_FEATURES=cuda
    else
      export BUILD_TARGET=cpu
      export DOCKER_FEATURES=
    fi
    docker compose --profile full build --no-cache
  fi
  
  echo -e "${GREEN}✓ Rebuild complete${NC}"
}

# Function to show status
show_status() {
  echo -e "${BLUE}📊 Service Status${NC}"
  echo ""
  
  if [ "$PLATFORM" = "macos" ]; then
    docker compose --profile core ps
    echo ""
    if pgrep -f "ninja-gekko" > /dev/null; then
      echo -e "${GREEN}✓ Native Ninja Gekko process is running${NC}"
    else
      echo -e "${YELLOW}⚠ Native Ninja Gekko process is not running${NC}"
    fi
  else
    docker compose --profile full ps
  fi
}

# Main command handler
case "${1:-start}" in
  start)
    check_docker
    
    if [ "$PLATFORM" = "macos" ]; then
      if check_metal; then
        start_macos_native
      else
        echo -e "${YELLOW}⚠ Metal not detected, falling back to CPU mode${NC}"
        start_cpu_mode
      fi
    else
      if check_nvidia_gpu; then
        start_cuda_mode
      else
        echo -e "${YELLOW}⚠ No NVIDIA GPU detected, using CPU mode${NC}"
        start_cpu_mode
      fi
    fi
    ;;
    
  stop)
    stop_services
    ;;
    
  logs)
    shift
    show_logs "$@"
    ;;
    
  rebuild)
    rebuild
    ;;
    
  status)
    show_status
    ;;
    
  *)
    echo "Usage: $0 {start|stop|logs|rebuild|status}"
    echo ""
    echo "Commands:"
    echo "  start    - Start the development environment"
    echo "  stop     - Stop all services"
    echo "  logs     - View logs (use 'logs [service]' for specific service)"
    echo "  rebuild  - Rebuild and restart services"
    echo "  status   - Show service status"
    exit 1
    ;;
esac

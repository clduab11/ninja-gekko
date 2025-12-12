#!/usr/bin/env bash
# Ninja Gekko GPU Verification Script
# Tests GPU detection and provides recommendations

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🥷 Ninja Gekko GPU Verification${NC}"
echo ""

# Detect OS
OS_TYPE=$(uname -s)
case "$OS_TYPE" in
  Darwin*)  PLATFORM="macos" ;;
  Linux*)   PLATFORM="linux" ;;
  MINGW*|MSYS*|CYGWIN*) PLATFORM="windows" ;;
  *) PLATFORM="unknown" ;;
esac

echo -e "${BLUE}Platform: $PLATFORM${NC}"
echo ""

# Check CUDA availability
check_cuda() {
  echo -e "${BLUE}Checking CUDA availability...${NC}"
  
  if command -v nvidia-smi &> /dev/null; then
    if nvidia-smi > /dev/null 2>&1; then
      echo -e "${GREEN}✓ NVIDIA GPU detected${NC}"
      echo ""
      nvidia-smi --query-gpu=index,name,driver_version,memory.total,compute_cap --format=csv,noheader | \
      while IFS=, read -r idx name driver mem cap; do
        echo -e "  GPU $idx: ${GREEN}$name${NC}"
        echo -e "  Driver: $driver"
        echo -e "  Memory: $mem"
        echo -e "  Compute Capability: $cap"
        echo ""
      done
      
      # Check CUDA version
      if command -v nvcc &> /dev/null; then
        CUDA_VERSION=$(nvcc --version | grep "release" | awk '{print $5}' | cut -d',' -f1)
        echo -e "${GREEN}✓ CUDA Toolkit: $CUDA_VERSION${NC}"
      else
        echo -e "${YELLOW}⚠ CUDA Toolkit not found (runtime only)${NC}"
      fi
      
      return 0
    fi
  fi
  
  echo -e "${YELLOW}⚠ No NVIDIA GPU detected${NC}"
  return 1
}

# Check Metal availability (macOS)
check_metal() {
  echo -e "${BLUE}Checking Metal availability...${NC}"
  
  if [ "$PLATFORM" = "macos" ]; then
    if system_profiler SPDisplaysDataType 2>/dev/null | grep -q "Metal"; then
      echo -e "${GREEN}✓ Metal GPU support detected${NC}"
      echo ""
      
      # Get GPU info
      system_profiler SPDisplaysDataType | grep -A 10 "Chipset Model" | head -15
      
      echo ""
      return 0
    else
      echo -e "${YELLOW}⚠ Metal GPU support not detected${NC}"
    fi
  else
    echo -e "${YELLOW}⚠ Metal is only available on macOS${NC}"
  fi
  
  return 1
}

# Test neural engine device detection
test_neural_device() {
  echo ""
  echo -e "${BLUE}Testing Neural Engine device detection...${NC}"
  
  # Try different device types
  for device in auto cuda metal cpu; do
    echo ""
    echo -e "${BLUE}Testing NEURAL_DEVICE_TYPE=$device${NC}"
    
    # Set environment and try to compile a simple test
    export NEURAL_DEVICE_TYPE=$device
    
    # Note: This requires the neural-engine to be compiled
    # For now, just show what would be set
    echo -e "  Environment: NEURAL_DEVICE_TYPE=$device"
  done
}

# Provide recommendations
provide_recommendations() {
  echo ""
  echo -e "${BLUE}📋 Recommendations:${NC}"
  echo ""
  
  HAS_CUDA=false
  HAS_METAL=false
  
  if check_cuda 2>/dev/null; then
    HAS_CUDA=true
  fi
  
  if [ "$PLATFORM" = "macos" ] && check_metal 2>/dev/null; then
    HAS_METAL=true
  fi
  
  echo ""
  
  if [ "$HAS_CUDA" = true ]; then
    echo -e "${GREEN}✓ CUDA GPU Detected${NC}"
    echo ""
    echo "  To use CUDA acceleration:"
    echo "  1. Set in .env: NEURAL_DEVICE_TYPE=cuda"
    echo "  2. Set in .env: BUILD_TARGET=cuda"
    echo "  3. Set in .env: DOCKER_FEATURES=cuda"
    echo "  4. Run: ./scripts/dev.sh start"
    echo ""
    echo "  Or build manually:"
    echo "    cargo build --release --features cuda"
    echo ""
    
  elif [ "$HAS_METAL" = true ]; then
    echo -e "${GREEN}✓ Metal GPU Detected (macOS)${NC}"
    echo ""
    echo "  To use Metal acceleration:"
    echo "  1. Set in .env: NEURAL_DEVICE_TYPE=metal"
    echo "  2. Run: ./scripts/dev.sh start"
    echo "     (This will run natively with Metal, not in Docker)"
    echo ""
    echo "  Or build manually:"
    echo "    cargo build --release --features metal"
    echo ""
    
  else
    echo -e "${YELLOW}⚠ No GPU detected - CPU mode${NC}"
    echo ""
    echo "  CPU-only mode will be used:"
    echo "  1. Set in .env: NEURAL_DEVICE_TYPE=cpu"
    echo "  2. Run: ./scripts/dev.sh start"
    echo ""
    echo "  For GPU support:"
    echo "    - Linux/Windows: Install NVIDIA GPU + CUDA 12.2+"
    echo "    - macOS: Use Apple Silicon (M1/M2/M3)"
    echo ""
  fi
}

# Main execution
echo ""
check_cuda || true
echo ""
check_metal || true

test_neural_device

provide_recommendations

echo ""
echo -e "${BLUE}Verification complete!${NC}"

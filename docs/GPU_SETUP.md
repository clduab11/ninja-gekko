# GPU Setup Guide for Ninja Gekko

This guide covers GPU acceleration setup for the Ninja Gekko neural engine across different platforms.

## Overview

Ninja Gekko supports hardware-accelerated neural inference using:
- **NVIDIA CUDA** (Linux/Windows)
- **Apple Metal** (macOS with Apple Silicon)
- **CPU fallback** (all platforms)

## Architecture

### macOS (Metal)
Docker Desktop for Mac cannot pass Metal GPU to Linux containers, so we use a **hybrid approach**:
- Infrastructure services (PostgreSQL, Redis, Prometheus, Grafana) run in Docker
- Ninja Gekko backend runs **natively** with Metal acceleration

### Linux/Windows (CUDA)
Full stack runs in Docker with NVIDIA GPU passthrough:
- All services run in containers
- Trading engine uses CUDA runtime base image
- GPU resources allocated via Docker Compose

## Platform-Specific Setup

### macOS with Apple Silicon (M1/M2/M3)

**Prerequisites:**
- macOS 12.0+ (Monterey or later)
- Apple Silicon Mac (M1/M2/M3)
- Docker Desktop for Mac
- Xcode Command Line Tools

**Installation:**

1. Install Xcode Command Line Tools:
   ```bash
   xcode-select --install
   ```

2. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

3. Configure environment in `.env`:
   ```bash
   NEURAL_DEVICE_TYPE=metal
   BUILD_TARGET=cpu  # Not used for native mode
   DOCKER_FEATURES=  # Not used for native mode
   ```

4. Start the system:
   ```bash
   ./scripts/dev.sh start
   ```

This will:
- Start PostgreSQL, Redis, Prometheus, Grafana in Docker
- Build Ninja Gekko natively with Metal support
- Run the trading engine with Metal GPU acceleration

**Verification:**
```bash
# Check Metal support
./scripts/verify_gpu.sh

# Check logs for Metal confirmation
# Look for: "🚀 Using device: Metal GPU"
```

### Linux with NVIDIA GPU

**Prerequisites:**
- NVIDIA GPU with Compute Capability 5.0+ (Maxwell generation or newer)
- NVIDIA Driver 525.60.13+ (for CUDA 12.2)
- Docker with NVIDIA Container Toolkit
- Ubuntu 20.04+ or equivalent

**Installation:**

1. Install NVIDIA drivers:
   ```bash
   # Ubuntu/Debian
   sudo apt update
   sudo apt install nvidia-driver-535
   
   # Verify installation
   nvidia-smi
   ```

2. Install NVIDIA Container Toolkit:
   ```bash
   distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
   curl -s -L https://nvidia.github.io/nvidia-docker/gpgkey | sudo apt-key add -
   curl -s -L https://nvidia.github.io/nvidia-docker/$distribution/nvidia-docker.list | \
     sudo tee /etc/apt/sources.list.d/nvidia-docker.list
   
   sudo apt-get update
   sudo apt-get install -y nvidia-container-toolkit
   sudo systemctl restart docker
   ```

3. Configure environment in `.env`:
   ```bash
   NEURAL_DEVICE_TYPE=cuda
   BUILD_TARGET=cuda
   DOCKER_FEATURES=cuda
   NVIDIA_GPU_COUNT=all  # Or specific GPU index: 0, 1, etc.
   ```

4. Start the system:
   ```bash
   ./scripts/dev.sh start
   ```

**Verification:**
```bash
# Check CUDA support
./scripts/verify_gpu.sh

# Check Docker GPU access
docker run --rm --gpus all nvidia/cuda:12.2.0-base-ubuntu22.04 nvidia-smi

# Check logs for CUDA confirmation
docker compose --profile full logs trading-engine | grep "Using device"
# Look for: "🚀 Using device: CUDA GPU 0"
```

### Windows with NVIDIA GPU

**Prerequisites:**
- Windows 10/11 with WSL2
- NVIDIA GPU with Compute Capability 5.0+
- NVIDIA Driver 525.60.13+ (for CUDA 12.2)
- Docker Desktop for Windows with WSL2 backend

**Installation:**

1. Install WSL2:
   ```powershell
   wsl --install
   wsl --set-default-version 2
   ```

2. Install NVIDIA Driver (Windows host):
   - Download from [NVIDIA Driver Downloads](https://www.nvidia.com/Download/index.aspx)
   - Install on Windows (NOT in WSL)

3. Install Docker Desktop with WSL2 integration

4. In WSL2, configure environment in `.env`:
   ```bash
   NEURAL_DEVICE_TYPE=cuda
   BUILD_TARGET=cuda
   DOCKER_FEATURES=cuda
   NVIDIA_GPU_COUNT=all
   ```

5. Start the system:
   ```bash
   ./scripts/dev.sh start
   ```

**Verification:**
```bash
# Check GPU from WSL
nvidia-smi

# Check CUDA support
./scripts/verify_gpu.sh
```

## Configuration Options

### Environment Variables

| Variable | Values | Description |
|----------|--------|-------------|
| `NEURAL_DEVICE_TYPE` | `auto`, `cuda`, `metal`, `cpu` | Device selection for neural inference |
| `NEURAL_BACKEND` | `candle`, `pytorch`, `cpu` | ML framework backend |
| `NEURAL_MODEL_PATH` | Path | Directory containing model files |
| `BUILD_TARGET` | `cpu`, `cuda` | Docker build target |
| `DOCKER_FEATURES` | `cuda`, `metal`, `candle`, `` | Rust feature flags for Docker build |
| `NVIDIA_GPU_COUNT` | `all`, `1`, `2`, etc. | Number of GPUs to allocate |

### Feature Flags

Build with specific features:

```bash
# Metal (macOS native)
cargo build --release --features metal

# CUDA (Linux/Windows native)
cargo build --release --features cuda

# CPU only
cargo build --release

# Candle with CPU
cargo build --release --features candle
```

## Performance Benchmarks

Expected inference latency improvements with GPU acceleration:

| Operation | CPU | CUDA (RTX 3090) | Metal (M2 Pro) |
|-----------|-----|-----------------|----------------|
| Volatility Prediction | 15-20ms | 2-3ms | 4-6ms |
| Cross-Exchange Analysis | 25-35ms | 4-6ms | 8-12ms |
| Risk Assessment | 10-15ms | 1-2ms | 3-4ms |
| Batch Inference (32) | 400-500ms | 50-70ms | 100-150ms |

*Benchmarks are approximate and depend on model size and complexity*

## Troubleshooting

### CUDA Issues

**Error: "CUDA initialization failed"**
```bash
# Check NVIDIA driver
nvidia-smi

# Check CUDA version compatibility
nvcc --version  # Should be 12.2+

# Verify Docker GPU access
docker run --rm --gpus all nvidia/cuda:12.2.0-base-ubuntu22.04 nvidia-smi
```

**Error: "libcuda.so not found"**
- Install NVIDIA Container Toolkit
- Restart Docker daemon
- Verify nvidia-docker2 package is installed

**Error: "Out of memory"**
```bash
# Reduce batch size in .env
NEURAL_INFERENCE_BATCH_SIZE=16

# Or limit GPU memory allocation
NVIDIA_VISIBLE_DEVICES=0  # Use specific GPU
```

### Metal Issues

**Error: "Metal device not available"**
```bash
# Verify Metal support
system_profiler SPDisplaysDataType | grep Metal

# Check macOS version (requires 12.0+)
sw_vers

# Ensure building with metal feature
cargo build --release --features metal
```

**Error: "accelerate-src linking failed"**
- Install Xcode Command Line Tools
- Update to latest macOS version
- Ensure Xcode license is accepted: `sudo xcodebuild -license accept`

### Docker Issues

**Error: "nvidia runtime not found"**
```bash
# Install NVIDIA Container Toolkit
sudo apt-get install -y nvidia-container-toolkit

# Restart Docker
sudo systemctl restart docker

# Test GPU access
docker run --rm --gpus all nvidia/cuda:12.2.0-base-ubuntu22.04 nvidia-smi
```

**Error: "profile core not found"**
- Update Docker Compose to v1.28.0+ or Docker Compose V2
- Profiles require newer Docker Compose versions

### Build Issues

**Error: "candle-core compilation failed"**
```bash
# For CUDA: Install CUDA Toolkit 12.2+
wget https://developer.download.nvidia.com/compute/cuda/12.2.0/local_installers/cuda_12.2.0_535.54.03_linux.run
sudo sh cuda_12.2.0_535.54.03_linux.run

# For Metal: Install Xcode Command Line Tools
xcode-select --install
```

## Model Optimization Tips

### CUDA Optimization
1. **Use tensor cores**: Enable mixed precision (FP16) for RTX GPUs
2. **Batch inference**: Increase `NEURAL_INFERENCE_BATCH_SIZE` for better throughput
3. **Model quantization**: Use INT8 quantization for 4x speedup with minimal accuracy loss
4. **Multiple GPUs**: Set `NVIDIA_GPU_COUNT=all` and use data parallelism

### Metal Optimization
1. **Unified memory**: Metal shares memory with CPU, reducing transfer overhead
2. **Metal Performance Shaders**: Candle-metal uses MPS for optimized ops
3. **Batch size**: Start with 16-32 for optimal M1/M2 performance
4. **Model caching**: Set `NEURAL_MODEL_CACHE_SIZE=4` to keep models in memory

## Monitoring GPU Usage

### NVIDIA GPUs
```bash
# Real-time monitoring
watch -n 1 nvidia-smi

# Detailed metrics
nvidia-smi dmon -s u

# Power and temperature
nvidia-smi pmon
```

### Apple Metal
```bash
# Activity Monitor > GPU tab
# Or use terminal:
sudo powermetrics --samplers gpu_power -i 1000 -n 10
```

## Advanced Configuration

### Multi-GPU Setup (NVIDIA)

To use specific GPUs:
```bash
# .env configuration
NVIDIA_GPU_COUNT=2          # Use 2 GPUs
CUDA_VISIBLE_DEVICES=0,1    # Specific GPU indices
```

### Custom Model Paths

```bash
# .env configuration
NEURAL_MODEL_PATH=/custom/path/to/models

# Ensure path is mounted in docker-compose.yml:
volumes:
  - /custom/path/to/models:/app/models:ro
```

### Performance Tuning

```bash
# .env configuration
NEURAL_INFERENCE_BATCH_SIZE=64     # Larger batches for throughput
NEURAL_MODEL_CACHE_SIZE=8          # Cache more models
NEURAL_DEVICE_TYPE=auto            # Auto-detect best device
```

## CI/CD Considerations

### GitHub Actions

CPU-only builds:
```yaml
- name: Build CPU
  run: cargo build --release
```

CUDA builds (requires self-hosted runners with GPU):
```yaml
- name: Build CUDA
  run: cargo build --release --features cuda
  if: runner.os == 'Linux'
```

### Docker Build Commands

```bash
# CPU build
docker build --build-arg BUILD_TARGET=cpu -t ninja-gekko:cpu .

# CUDA build
docker build --build-arg BUILD_TARGET=cuda --build-arg FEATURES=cuda -t ninja-gekko:cuda .

# Multi-platform build (CPU only, for ARM64 + AMD64)
docker buildx build --platform linux/amd64,linux/arm64 -t ninja-gekko:latest .
```

## Support

For GPU-related issues:
1. Run `./scripts/verify_gpu.sh` and include output in bug reports
2. Check logs for device initialization messages
3. Verify driver/toolkit versions match requirements
4. Review Docker GPU access with test containers

## References

- [Candle ML Framework](https://github.com/huggingface/candle)
- [NVIDIA Container Toolkit](https://github.com/NVIDIA/nvidia-docker)
- [Metal Performance Shaders](https://developer.apple.com/metal/)
- [CUDA Toolkit Documentation](https://docs.nvidia.com/cuda/)

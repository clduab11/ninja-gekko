# scripts/dev-setup-windows.ps1
# Helper script to launch Ninja Gekko on Windows (WSL2) with CUDA support

Write-Host "🦎 Ninja Gekko - Windows CUDA Setup"
Write-Host "=================================="

# Check for .env file
if (-not (Test-Path ".env")) {
    Write-Host "Creating .env from template..."
    Copy-Item ".env.template" -Destination ".env"
}

Write-Host "Setting up environment for CUDA mode..."

# Set environment variables for the session
$Env:BUILD_TARGET = "cuda"
$Env:COMPOSE_PROFILES = "full"
$Env:DOCKER_DEFAULT_PLATFORM = "linux/amd64" # Standard for CUDA containers

Write-Host "Building containers (Target: CUDA)..."
# We primarily need to ensure trading-engine is built with cuda target
docker compose build --build-arg BUILD_TARGET=cuda trading-engine
docker compose build frontend

Write-Host "Starting services with CUDA support..."
Write-Host "Run 'docker compose -f docker-compose.yml -f docker-compose.cuda.yml up -d' to start in background."

# Launch with both base config and CUDA override
docker compose -f docker-compose.yml -f docker-compose.cuda.yml up -d trading-engine frontend postgres redis

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Services started successfully."
    Write-Host "   Frontend: http://localhost:5173"
    Write-Host "   Backend:  http://localhost:8080"
} else {
    Write-Host "❌ Setup failed."
}

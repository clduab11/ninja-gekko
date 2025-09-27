# Ninja Gekko Deployment Playbooks

This guide documents reproducible deployment flows for the Ninja Gekko autonomous trading stack across containerized and Kubernetes environments. All steps assume access to the Tenno-MCP tool registry and the neural benchmarking harness introduced in the README.

## Docker Compose Playbook

1. **Prerequisites**
   - Docker Engine 24+
   - NVIDIA Container Toolkit (for GPU inference nodes)
   - Access credentials for supported exchanges and data providers stored in `.env`
2. **Bootstrap**
   - Clone the repository and run `cargo build --release` for the core services if building locally.
   - Copy the template below into `deploy/docker-compose.yml` (or another filename of your choosing).

```yaml
services:
  tenno-mcp-router:
    image: ghcr.io/ninja-gekko/tenno-mcp:latest
    env_file: .env
    ports:
      - "8080:8080"
  trading-engine:
    build:
      context: ..
      dockerfile: Dockerfile
    command: ["./target/release/ninja-gekko", "--mode", "swarm"]
    depends_on:
      - tenno-mcp-router
  gpu-inference:
    image: ghcr.io/ninja-gekko/neural-inference:latest
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: ["gpu"]
  observability:
    image: ghcr.io/ninja-gekko/observability:latest
    ports:
      - "3000:3000"
```

   - Execute `docker compose -f deploy/docker-compose.yml up -d` to launch the trading engine, Tenno-MCP router, observability stack, and GPU inference workers.
3. **Post-Launch Validation**
   - Confirm Tenno-MCP service discovery via `curl http://localhost:8080/mcp/health`.
   - Review Prometheus and Grafana dashboards at `http://localhost:3000` for latency, Sharpe, and uptime metrics.
   - Run the neural benchmarking harness with `cargo run --bin benchmarking -- --suite neural-default` to populate baseline metrics.

## Kubernetes Guide

1. **Cluster Requirements**
   - Kubernetes 1.28+
   - GPU-enabled node pool with NVIDIA device plugin or equivalent
   - Istio or Linkerd for secure service mesh routing
2. **Installation Steps**
   - Create the following Kustomize structure (paths relative to the repository root):

```text
deploy/
  k8s/
    base/
      kustomization.yaml
      tenno-mcp-router.yaml
      trading-engine.yaml
    gpu/
      kustomization.yaml
      gpu-inference.yaml
    hpa/
      kustomization.yaml
      trading-engine-hpa.yaml
    benchmarks.yaml
```

   - Populate the manifests using the examples below, then apply them with `kubectl apply -k deploy/k8s/base`, `kubectl apply -k deploy/k8s/gpu`, and `kubectl apply -k deploy/k8s/hpa`.
   - Configure Istio gateway and mTLS policies to secure MCP and trading ingress points.

```yaml
# deploy/k8s/base/tenno-mcp-router.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: tenno-mcp-router
spec:
  replicas: 2
  selector:
    matchLabels:
      app: tenno-mcp-router
  template:
    metadata:
      labels:
        app: tenno-mcp-router
    spec:
      containers:
        - name: router
          image: ghcr.io/ninja-gekko/tenno-mcp:latest
          ports:
            - containerPort: 8080
          envFrom:
            - secretRef:
                name: tenno-mcp-secrets
---
# deploy/k8s/base/trading-engine.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: trading-engine
spec:
  replicas: 3
  selector:
    matchLabels:
      app: trading-engine
  template:
    metadata:
      labels:
        app: trading-engine
    spec:
      containers:
        - name: engine
          image: ghcr.io/ninja-gekko/trading-engine:latest
          args: ["--mode", "swarm"]
          envFrom:
            - secretRef:
                name: trading-credentials
---
# deploy/k8s/gpu/gpu-inference.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: gpu-inference
spec:
  replicas: 1
  selector:
    matchLabels:
      app: gpu-inference
  template:
    metadata:
      labels:
        app: gpu-inference
    spec:
      containers:
        - name: inference
          image: ghcr.io/ninja-gekko/neural-inference:latest
          resources:
            limits:
              nvidia.com/gpu: 1
---
# deploy/k8s/hpa/trading-engine-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: trading-engine-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: trading-engine
  minReplicas: 3
  maxReplicas: 15
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 60
```
3. **Operational Checks**
   - Validate swarm agent registration through `kubectl logs deployment/tenno-mcp-router`.
   - Inspect OpenTelemetry traces for end-to-end order lifecycles in Jaeger/Grafana Tempo.
   - Schedule the benchmarking CronJob (`kubectl apply -f deploy/k8s/benchmarks.yaml`) to publish inference latency reports.

## Benchmark Automation

- Continuous integration workflows invoke `cargo run --bin benchmarking -- --suite regression` on each merge to the `main` branch.
- Results are exported to Prometheus via the `benchmark-exporter` sidecar and visualized in Grafana (`dashboards/benchmarks.json`).
- To compare against Neural Trader MCP or Claude-Flow baselines, import their metrics into the `comparison` data source and re-run the harness with the `--peer neural-trader` or `--peer claude-flow` flags.

## Sandbox Parity Checklist

- ✅ Tenno-MCP tool registry lists Playwright, Filesystem, GitHub, Supabase, and Search MCP services.
- ✅ Neural benchmarking manifests match the versions tracked in your environment-specific configuration repository.
- ✅ Docker and Kubernetes deployments emit identical telemetry labels, enabling shared dashboards and alerting rules.

> **Reminder:** All deployments must be exercised in paper-trading or sandbox environments before real-capital execution. Adjust rate limits, risk controls, and compliance hooks per jurisdictional requirements.

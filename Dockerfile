FROM rust:latest
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY . .
# Disable sqlx compile-time checks
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin ninja-gekko

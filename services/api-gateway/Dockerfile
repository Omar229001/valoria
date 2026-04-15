# Image complète (pas slim) : a gcc, pkg-config, libssl-dev
FROM rust:latest AS builder
WORKDIR /app

# 1. Cache des dépendances
COPY Cargo.toml .
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/api_gateway* target/release/api-gateway

# 2. Build réel
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Image finale légère
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/api-gateway .
EXPOSE 8080
CMD ["./api-gateway"]

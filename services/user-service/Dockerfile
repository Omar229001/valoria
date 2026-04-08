# Image complète : a gcc, pkg-config, libssl-dev
FROM rust:latest AS builder
WORKDIR /app

# 1. Cache des dépendances
COPY Cargo.toml .
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/user_service* target/release/user-service

# 2. Build réel
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Image finale légère
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/user-service .
EXPOSE 8000
CMD ["./user-service"]

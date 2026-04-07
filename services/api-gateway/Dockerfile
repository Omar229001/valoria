FROM rust:1.77-slim AS builder
WORKDIR /app
COPY Cargo.toml .
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/api-gateway .
EXPOSE 8080
CMD ["./api-gateway"]
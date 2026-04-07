FROM rust:1.77-slim AS builder
WORKDIR /app
COPY Cargo.toml .
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/user-service .
EXPOSE 8081
CMD ["./user-service"]

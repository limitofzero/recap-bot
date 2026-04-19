# Stage 1: Build
FROM rust:1.95 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./

COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/recap-bot /app/recap-bot
# EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info

# Run the bot
CMD ["./recap-bot"]

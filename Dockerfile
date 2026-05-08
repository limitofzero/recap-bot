FROM rust:1.95 AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/* \
    && useradd -r -u 1000 -m botuser

WORKDIR /app

COPY --from=builder /app/target/release/recap-bot /app/recap-bot

USER botuser
# EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info

# Run the bot
CMD ["./recap-bot"]

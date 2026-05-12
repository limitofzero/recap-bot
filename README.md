# Recap Bot

Telegram-бот на Rust для сохранения сообщений в чатах и (скоро) генерации саммари по запросу.

## Stack

- **Rust 1.95+** with **Tokio** async runtime
- **[teloxide](https://github.com/teloxide/teloxide)** — Telegram Bot API client
- **[sqlx](https://github.com/launchbadge/sqlx)** + **Postgres 16** — message storage, compile-time checked queries
- **[axum](https://github.com/tokio-rs/axum)** — health + metrics HTTP endpoints
- **[metrics](https://crates.io/crates/metrics)** + **Prometheus** — RED-style observability

## Architecture

Layered architecture: thin handlers → services orchestrate → repositories / clients do IO.

```
src/
├── main.rs              # composition root: build pool, dispatcher, run
├── app.rs               # AppState — wired dependencies passed via dptree
│
├── handlers/            # Telegram update handlers (thin)
│   ├── mod.rs
│   └── message.rs       # generic message handler — calls services::messages::save
│
├── services/            # business logic, framework-agnostic
│   ├── mod.rs
│   └── messages.rs      # save(): extract fields → upsert chat → upsert message
│
├── repositories/        # SQL only (sqlx queries)
│   ├── mod.rs
│   ├── chats.rs         # upsert chat
│   └── messages.rs      # upsert message
│
├── domain/              # plain types (Message DTOs, Command enum)
├── errors.rs            # AppError enum (thiserror)
├── health/              # /health + /metrics axum router
└── ...                  # future: clients/ (z.ai), handlers/commands/ (recap)
```

**Dependency rule**: handlers → services → repositories / clients. Never the other way around.

## Local development

### Prerequisites

- Rust toolchain (`rustup` install)
- Docker (for local Postgres)
- A **dev** Telegram bot token from [@BotFather](https://t.me/botfather) (do NOT reuse a production token — see "Telegram long-polling" below)

### Quick start

```bash
# 1. Postgres
docker compose up -d postgres

# 2. Env vars
cp .env.example .env
# Edit .env: put your dev TELOXIDE_TOKEN; DATABASE_URL is fine as-is
$EDITOR .env

# 3. Run
cargo run
```

The bot:
- Connects to Postgres on `localhost:5432`
- Runs migrations from `./migrations`
- Exposes `/health` and `/metrics` on `:8080`
- Connects to Telegram via long-polling

### Telegram long-polling note

⚠️ **Only one client can poll a bot token at a time.** If you run a copy locally AND in production with the same token, they fight ("TerminatedByOtherGetUpdates"). Always use a **separate dev bot** for local work.

### Useful commands

```bash
cargo check                                   # fast type check
cargo clippy --all-targets --all-features     # lints (fails CI if warnings)
cargo fmt                                     # format
cargo sqlx prepare -- --bin recap-bot         # refresh .sqlx offline cache after SQL changes
```

## Migrations

```bash
# install once
cargo install sqlx-cli --no-default-features --features postgres

# add a new migration
sqlx migrate add <name>

# run pending migrations
sqlx migrate run

# revert latest
sqlx migrate revert
```

Migrations live in `./migrations` and run automatically on bot startup (`migrate!()` macro).

## Observability

- **`/health`** — HTTP `GET`, returns 200 if `SELECT 1` against Postgres passes, 503 otherwise. Used by Kubernetes readiness probe.
- **`/metrics`** — Prometheus exposition format. Scraped by `kube-prometheus-stack` Prometheus via `ServiceMonitor`.

### Current metrics

| Metric | Type | Labels |
|---|---|---|
| `bot_messages_received_total` | counter | — |
| `bot_db_query_seconds` | histogram (summary) | `operation` ∈ `{insert_chat, insert_message}` |

## Deployment

Production deploy targets a single-node **k3s** cluster. CI/CD lives in `.github/workflows/`:

- **`ci.yml`** — lint + sqlx offline cache check on every PR / push to main
- **`build-image.yml`** — build & push Docker image to GHCR on push to main
- **`deploy.yml`** — applies k8s manifests via SSH, rolls out the Deployment

Kubernetes manifests in `.github/k8s/`:

```
.github/k8s/
├── namespace.yaml           # recap-bot namespace
├── postgres.yaml            # Headless Service + StatefulSet (10Gi PVC)
├── deployment.yaml          # bot Deployment (1 replica, probes, resources)
├── service.yaml             # ClusterIP Service (for Prometheus to scrape /metrics)
└── service-monitor.yaml     # Prometheus Operator CRD
```

Required GitHub Secrets (in `production` environment):

| Secret | Use |
|---|---|
| `SERVER_HOST` | k3s node public IP |
| `SERVER_USER` | SSH user on the server |
| `SERVER_SSH_KEY` | private key (matches deploy key in `authorized_keys`) |
| `POSTGRES_PASSWORD` | random 24-char password, set once |
| `TELOXIDE_TOKEN` | production bot token |

## License

(Add your license here)

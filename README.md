# Recap Bot

A Rust-based Telegram bot that automatically generates summaries (recaps) of conversations from Telegram channels and group chats.

## Overview

Recap Bot processes messages from Telegram channels and chats to create concise summaries of discussions, key points, and important information. Perfect for keeping track of large channels or busy group conversations without manually reading every message.

## Features

- 🤖 Automated recap generation from Telegram messages
- 📊 Summarizes channel/chat discussions
- ⚡ Built with Rust for performance and reliability
- 🔧 Configurable recap parameters

## Getting Started

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Cargo (comes with Rust)
- Telegram Bot Token (get one from [@BotFather](https://t.me/botfather) on Telegram)

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd recap-bot
```

2. Build the project:
```bash
cargo build --release
```

3. Configure your Telegram Bot Token (via environment variable or config file)

4. Run the bot:
```bash
cargo run --release
```

## Usage

(Add usage instructions once bot features are implemented)

## Development

### Build and Test

```bash
# Check code without building
cargo check

# Build debug version
cargo build

# Run tests
cargo test

# Run clippy linter
cargo clippy

# Format code
cargo fmt
```

### Project Structure

```
src/
├── main.rs          # Entry point and main logic
└── ...              # Additional modules as needed
```

## Dependencies

Key dependencies (see `Cargo.toml` for full list):
- **tokio** - Async runtime
- **teloxide** - Telegram Bot API client (if used)
- **serde** - Serialization/deserialization

## Contributing

Contributions are welcome! Please ensure code passes `cargo clippy` and `cargo fmt`.

## License

(Add your license here)

## Support

For issues and questions, please open an issue on the repository.

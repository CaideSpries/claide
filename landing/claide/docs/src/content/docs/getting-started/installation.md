---
title: Installation
description: Install Claide on your system
tableOfContents:
  minHeadingLevel: 2
  maxHeadingLevel: 4
---

Claide is distributed as a single static binary. Choose the installation method that works best for your platform.

## Prerequisites

- **macOS or Linux** — Claide runs on both platforms (x86_64 and ARM64)
- **No runtime dependencies** — The binary is fully self-contained

## Install with script (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/qhkm/claide/main/install.sh | sh
```

This downloads the latest release binary for your platform and places it in your PATH.

## Install with Homebrew (macOS/Linux)

```bash
brew install qhkm/tap/claide
```

## Install with Cargo

Build from source using Rust's package manager:

```bash
cargo install claide --git https://github.com/qhkm/claide
```

## Docker

Run Claide in a container:

```bash
docker pull ghcr.io/qhkm/claide:latest

# Run agent mode
docker run --rm ghcr.io/qhkm/claide:latest agent -m "Hello"

# Run gateway mode with config
docker run -d \
  -v ~/.claide:/root/.claide \
  -e CLAIDE_PROVIDERS_ANTHROPIC_API_KEY=sk-... \
  ghcr.io/qhkm/claide:latest gateway
```

## Download binary

Pre-built binaries are available on the [releases page](https://github.com/qhkm/claide/releases):

```bash
# Linux x86_64
curl -L https://github.com/qhkm/claide/releases/latest/download/claide-linux-x86_64 -o claide
chmod +x claide

# macOS (Apple Silicon)
curl -L https://github.com/qhkm/claide/releases/latest/download/claide-macos-aarch64 -o claide
chmod +x claide

# macOS (Intel)
curl -L https://github.com/qhkm/claide/releases/latest/download/claide-macos-x86_64 -o claide
chmod +x claide
```

## Build from source

To build from source, you need Rust 1.70+:

```bash
git clone https://github.com/qhkm/claide.git
cd claide

# Build release binary (~4MB)
cargo build --release

# Verify
./target/release/claide --version
```

## Verify installation

```bash
claide --version
# claide 0.5.0

claide --help
# Shows available commands
```

## Next steps

Now that Claide is installed, follow the [quick start guide](/docs/getting-started/quick-start/) to run your first agent interaction.

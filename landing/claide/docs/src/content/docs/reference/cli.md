---
title: CLI Reference
description: Complete command reference for the Claide CLI
tableOfContents:
  minHeadingLevel: 2
  maxHeadingLevel: 3
---

Claide uses a subcommand-based CLI built with [clap](https://docs.rs/clap).

## Global options

```
claide [OPTIONS] <COMMAND>
```

| Option | Description |
|--------|-------------|
| `--help` | Show help message |
| `--version` | Show version |

## agent

Run a single agent interaction.

```bash
claide agent [OPTIONS] -m <MESSAGE>
```

| Option | Description |
|--------|-------------|
| `-m, --message <TEXT>` | Message to send to the agent |
| `--stream` | Enable streaming (token-by-token output) |
| `--template <NAME>` | Use an agent template (coder, researcher, writer, analyst) |
| `--workspace <PATH>` | Set workspace directory |

### Examples

```bash
# Simple message
claide agent -m "Hello"

# With streaming
claide agent --stream -m "Explain async Rust"

# With template
claide agent --template coder -m "Write a CSV parser"
```

## gateway

Start the multi-channel gateway.

```bash
claide gateway [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--containerized [RUNTIME]` | Enable container isolation (auto, docker, apple) |
| `--tunnel [PROVIDER]` | Enable tunnel (auto, cloudflare, ngrok, tailscale) |

### Examples

```bash
# Start gateway
claide gateway

# With container isolation
claide gateway --containerized docker
```

## batch

Process multiple prompts from a file.

```bash
claide batch [OPTIONS] --input <FILE>
```

| Option | Description |
|--------|-------------|
| `--input <FILE>` | Input file (text or JSONL) |
| `--output <FILE>` | Output file (default: stdout) |
| `--format <FORMAT>` | Output format: text, jsonl |
| `--template <NAME>` | Agent template to use |
| `--stream` | Enable streaming per prompt |
| `--stop-on-error` | Stop on first error |

### Examples

```bash
# Process text file
claide batch --input prompts.txt

# JSONL output
claide batch --input prompts.txt --format jsonl --output results.jsonl

# With template and error handling
claide batch --input prompts.jsonl --template researcher --stop-on-error
```

## config check

Validate configuration file.

```bash
claide config check
```

Reports unknown fields, missing required values, and type errors.

## history

Manage conversation history.

```bash
claide history <SUBCOMMAND>
```

### history list

```bash
claide history list [--limit <N>]
```

List recent sessions with timestamps and titles.

### history show

```bash
claide history show <QUERY>
```

Show a session by fuzzy-matching the query against session titles and keys.

### history cleanup

```bash
claide history cleanup [--keep <N>]
```

Remove old sessions, keeping the most recent N (default: 50).

## template

Manage agent templates.

```bash
claide template <SUBCOMMAND>
```

### template list

List all available templates (built-in and custom).

### template show

```bash
claide template show <NAME>
```

Show template details including system prompt, model, and overrides.

## onboard

Run the interactive setup wizard.

```bash
claide onboard [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--full` | Run the full 10-step wizard instead of express setup |

Walks through provider key setup, channel configuration, and workspace initialization.

## heartbeat

View heartbeat service status.

```bash
claide heartbeat --show
```

## skills

Manage agent skills.

```bash
claide skills list
```

List available skills from `~/.claide/skills/`.

## secrets

Manage secret encryption at rest.

```bash
claide secrets <SUBCOMMAND>
```

### secrets encrypt

Encrypt plaintext API keys and tokens in your config file using XChaCha20-Poly1305.

```bash
claide secrets encrypt
```

### secrets decrypt

Decrypt secrets for editing.

```bash
claide secrets decrypt
```

### secrets rotate

Re-encrypt with a new master key.

```bash
claide secrets rotate
```

## memory

Manage long-term memory from the CLI.

```bash
claide memory <SUBCOMMAND>
```

### memory list

```bash
claide memory list [--category <CATEGORY>]
```

### memory search

```bash
claide memory search <QUERY>
```

### memory set

```bash
claide memory set <KEY> <VALUE> [--category <CATEGORY>] [--tags <TAGS>]
```

### memory delete

```bash
claide memory delete <KEY>
```

### memory stats

```bash
claide memory stats
```

## tools

Discover available tools.

```bash
claide tools <SUBCOMMAND>
```

### tools list

List all available tools and their status.

```bash
claide tools list
```

### tools info

Show detailed info about a specific tool.

```bash
claide tools info <NAME>
```

## watch

Monitor a URL for changes.

```bash
claide watch <URL> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--interval <DURATION>` | Check interval (e.g., 1h, 30m) |
| `--notify <CHANNEL>` | Channel for notifications |

## channel

Manage channels.

```bash
claide channel <SUBCOMMAND>
```

### channel list

```bash
claide channel list
```

### channel setup

```bash
claide channel setup <NAME>
```

### channel test

```bash
claide channel test <NAME>
```

## migrate

Import config and skills from an OpenClaw installation.

```bash
claide migrate [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--from <PATH>` | Path to OpenClaw installation |
| `--dry-run` | Preview migration without writing files |
| `--yes` | Non-interactive (skip prompts) |

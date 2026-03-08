---
title: Deployment
description: Deploy Claide to production
tableOfContents:
  minHeadingLevel: 2
  maxHeadingLevel: 3
---

Claide can be deployed anywhere a Linux binary can run. Choose the method that fits your infrastructure.

## One-click deploy

### Any VPS

The fastest way to deploy Claide is using the automated setup script:

```bash
curl -fsSL https://claide.com/setup.sh | bash
```

This interactive wizard will:
- Download the latest Claide binary
- Guide you through configuring your LLM provider (Anthropic or OpenAI)
- Set up your messaging channel (Telegram, Slack, Discord, or Webhook)
- Install and start a systemd service

**Docker deployment:**
```bash
curl -fsSL https://claide.com/setup.sh | bash -s -- --docker
```

**Uninstall:**
```bash
curl -fsSL https://claide.com/setup.sh | bash -s -- --uninstall
```

## Docker (single container)

The simplest production deployment:

```dockerfile
FROM ghcr.io/qhkm/claide:latest

COPY config.json /root/.claide/config.json

EXPOSE 8080
CMD ["claide", "gateway"]
```

```bash
docker build -t my-agent .
docker run -d --name claide \
  -e CLAIDE_PROVIDERS_ANTHROPIC_API_KEY=sk-ant-... \
  -e CLAIDE_CHANNELS_TELEGRAM_BOT_TOKEN=123456:ABC... \
  my-agent
```

## Docker Compose

For multi-service setups:

```yaml
version: '3.8'
services:
  claide:
    image: ghcr.io/qhkm/claide:latest
    restart: unless-stopped
    environment:
      - CLAIDE_PROVIDERS_ANTHROPIC_API_KEY=${ANTHROPIC_KEY}
      - CLAIDE_CHANNELS_TELEGRAM_BOT_TOKEN=${TELEGRAM_TOKEN}
    volumes:
      - claide-data:/root/.claide
    healthcheck:
      test: ["CMD", "claide", "config", "check"]
      interval: 30s
      retries: 3

volumes:
  claide-data:
```

## Fly.io

Deploy to Fly.io with a single command:

```toml
# fly.toml
app = "my-claide"
primary_region = "sin"

[build]
  image = "ghcr.io/qhkm/claide:latest"

[env]
  RUST_LOG = "info"

[[services]]
  internal_port = 8080
  protocol = "tcp"

  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]
```

```bash
fly launch
fly secrets set CLAIDE_PROVIDERS_ANTHROPIC_API_KEY=sk-ant-...
fly secrets set CLAIDE_CHANNELS_TELEGRAM_BOT_TOKEN=123456:ABC...
fly deploy
```

## Railway

Deploy via Railway CLI:

```bash
railway init
railway up

# Set secrets
railway variables set CLAIDE_PROVIDERS_ANTHROPIC_API_KEY=sk-ant-...
railway variables set CLAIDE_CHANNELS_TELEGRAM_BOT_TOKEN=123456:ABC...
```

## Render

Use the Render dashboard or `render.yaml`:

```yaml
services:
  - type: worker
    name: claide
    runtime: docker
    dockerfilePath: ./Dockerfile
    envVars:
      - key: CLAIDE_PROVIDERS_ANTHROPIC_API_KEY
        sync: false
      - key: CLAIDE_CHANNELS_TELEGRAM_BOT_TOKEN
        sync: false
```

## Systemd (bare metal)

For direct deployment on a Linux server:

```ini
# /etc/systemd/system/claide.service
[Unit]
Description=Claide AI Agent
After=network-online.target

[Service]
Type=simple
User=claide
ExecStart=/usr/local/bin/claide gateway
Restart=always
RestartSec=5
Environment=RUST_LOG=info
EnvironmentFile=/etc/claide/env

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable claide
sudo systemctl start claide
```

## Health checks

Claide's `config check` command can be used as a health check:

```bash
claide config check
```

Returns exit code 0 if configuration is valid.

## Persistent data

Important directories to persist across restarts:

| Path | Contents |
|------|----------|
| `~/.claide/config.json` | Configuration |
| `~/.claide/memory/` | Long-term memory |
| `~/.claide/sessions/` | Conversation history |
| `~/.claide/skills/` | Custom skills |
| `~/.claide/plugins/` | Custom plugins |

Mount `~/.claide` as a volume in Docker deployments.

## Resource requirements

Claide is very lightweight:

| Resource | Requirement |
|----------|-------------|
| CPU | Any modern CPU (ARM64 or x86_64) |
| RAM | ~6MB RSS at idle |
| Disk | ~4MB binary + data |
| Network | Outbound HTTPS to LLM provider APIs |

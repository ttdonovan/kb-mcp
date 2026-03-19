# Researcher Agent

A containerized agent that uses kb-mcp to discover new content about AI
agent memory and curate it into the vault.

## Prerequisites

- Docker + Docker Compose
- Host Ollama (dev) or Anthropic/OpenAI API key (prod)
- Web search uses DuckDuckGo — no API key needed

## Setup

```sh
# 1. Copy provider config
cp agents/researcher/config/config.toml.ollama.example agents/researcher/config/config.toml

# 2. (Optional) Create .env for cloud LLM provider
echo "ANTHROPIC_API_KEY=sk-ant-..." > .env

# 3. Build the container
just agent-build
```

## Usage

```sh
# Interactive research session
just agent-research

# Research a specific topic
just agent-research-topic "HNSW vector search performance"

# Check what the vault contains
just agent-vault-status
```

## How It Works

1. Agent receives a research topic (manual prompt)
2. Searches existing vault via kb-mcp (avoids duplicates)
3. Searches the web via DuckDuckGo (Earl template)
4. Fetches and reads promising sources
5. Synthesizes a vault entry with proper frontmatter + source citations
6. Writes to the vault via `kb_write`
7. You review the new entry on the host and `git commit`

## Container Security

- Read-only root filesystem
- Non-root user (uid 1001)
- Named volume for runtime workspace
- IDENTITY.md and SOUL.md mounted read-only
- tmpfs for temp files (64MB cap)
- Localhost-only ports
- API keys via environment variables (not baked into image)

## Agent Identity

- **IDENTITY.md** — defines the research domain, available tools, and boundaries
- **SOUL.md** — quality principles: primary sources first, cite everything, flag uncertainty

## Resources

- [ZeroClaw Documentation](https://github.com/zeroclaw-labs/zeroclaw/blob/master/docs/README.md)
- [kb-mcp](https://github.com/ttdonovan/kb-mcp) — the knowledge base MCP server this agent uses
- [ddg-web-search](https://clawhub.ai/JakeLin/ddg-web-search) — DuckDuckGo search skill (inspiration)

---
title: "feat: Container researcher agent for vault curation"
type: feat
status: completed
date: 2026-03-19
origin: docs/brainstorms/2026-03-19-container-researcher-agent-brainstorm.md
---

# feat: Container researcher agent for vault curation

## Overview

Add a containerized research agent that uses kb-mcp to discover new
content about AI agent memory and curate it into the vault. Runs in a
single ZeroClaw container with web search via Earl. Manually triggered.

This is the first consumer of kb-mcp beyond Claude Code sessions — it
dogfoods the tool in a containerized, autonomous context.

## Problem Statement / Motivation

The vault about AI agent memory needs to stay current. New papers, tools,
and projects appear regularly. Manually tracking and writing entries is
the bottleneck. A research agent can automate discovery and first-draft
creation, leaving human review as the quality gate.

This also validates kb-mcp's design in a real containerized deployment —
every rough edge we hit is a product improvement.

(See brainstorm: `docs/brainstorms/2026-03-19-container-researcher-agent-brainstorm.md`)

## Proposed Solution

A single ZeroClaw container with kb-mcp + Earl built from source. The
vault is volume-mounted for read-write access. The agent searches the
web, evaluates relevance against existing vault content, and writes new
entries via `kb_write`. Human reviews and git-commits from the host.

```
┌─────────────────────────────────────────┐
│  ZeroClaw Container                     │
│                                         │
│  IDENTITY.md + SOUL.md (read-only)      │
│  kb-mcp (MCP server → vault)            │
│  Earl (web search + fetch)              │
│                                         │
│  vault/ ←──── volume mount (read-write) │
└─────────────────────────────────────────┘
         │                    ▲
         │ web search         │ new .md files
         ▼                    │
    Brave Search API    Host filesystem
                         └── git commit
```

## Technical Approach

### Directory Structure

```
agents/
└── researcher/
    ├── config/
    │   ├── config.toml.ollama.example
    │   └── config.toml.anthropic.example
    └── workspace/
        ├── IDENTITY.md
        ├── SOUL.md
        └── collections.ron

templates/
├── http.web-search.hcl
└── http.web-fetch.hcl

Dockerfile
docker-compose.yml
```

### Dockerfile (Multi-Stage)

Following a proven production pattern — three build stages + runtime:

```dockerfile
# Stage 1: Build ZeroClaw
FROM rust:latest AS zeroclaw-builder
RUN cargo install zeroclaw

# Stage 2: Build kb-mcp
FROM rust:latest AS kbmcp-builder
RUN cargo install --git https://github.com/ttdonovan/kb-mcp.git

# Stage 3: Build Earl (minimal features)
FROM rust:latest AS earl-builder
RUN cargo install earl --no-default-features --features http

# Stage 4: Runtime
FROM debian:trixie-slim
COPY --from=zeroclaw-builder /usr/local/cargo/bin/zeroclaw /usr/local/bin/
COPY --from=kbmcp-builder /usr/local/cargo/bin/kb-mcp /usr/local/bin/
COPY --from=earl-builder /usr/local/cargo/bin/earl /usr/local/bin/

RUN useradd -u 1001 -m agent
USER agent
WORKDIR /workspace
```

### Docker Compose

```yaml
services:
  researcher:
    build: .
    profiles: [dev]
    volumes:
      - researcher-workspace:/workspace
      - ./vault:/workspace/vault
      - ./agents/researcher/workspace/IDENTITY.md:/workspace/IDENTITY.md:ro
      - ./agents/researcher/workspace/SOUL.md:/workspace/SOUL.md:ro
      - ./agents/researcher/workspace/collections.ron:/workspace/collections.ron:ro
    read_only: true
    tmpfs:
      - /tmp:size=64M
    ports:
      - "127.0.0.1:42710:42710"
    extra_hosts:
      - "host-gateway:host-gateway"
    environment:
      - BRAVE_API_KEY=${BRAVE_API_KEY}

volumes:
  researcher-workspace:
```

### Agent Identity

**IDENTITY.md** — defines purpose, tools, boundaries:
- Knowledge researcher specializing in AI agent memory
- Available tools: kb-mcp (search, context, write, reindex), Earl (web-search, web-fetch)
- Boundaries: only writes to vault via kb_write, never modifies existing files,
  sources every claim, searches existing content before writing to avoid duplicates

**SOUL.md** — decision principles:
- Prefer primary sources (papers, official docs) over blog posts
- Always include source URLs in frontmatter `sources:` field
- Flag uncertainty rather than fabricate ("reportedly" vs asserting)
- One topic per file, properly tagged and sectioned
- Quality over quantity — one well-sourced entry beats five shallow ones

### collections.ron (Inside Container)

```ron
(
    collections: [
        (
            name: "vault",
            path: "/workspace/vault",
            description: "AI agent memory knowledge vault",
            writable: true,
            sections: [
                (prefix: "concepts", description: "Memory models and retrieval strategies"),
                (prefix: "patterns", description: "Knowledge keeper, session digests, shared memory"),
                (prefix: "tools", description: "Retrieval tool landscape"),
                (prefix: "research", description: "Papers, projects, reading list"),
            ],
        ),
    ],
)
```

### Earl Templates

Two minimal templates for web research:

**`http.web-search.hcl`** — Brave Search API:
```hcl
command "web-search" {
  description = "Search the web for a topic"
  mode        = "read"

  param "query" { type = "string"; required = true }
  param "count" { type = "number"; default = 10 }

  operation {
    protocol = "http"
    method   = "GET"
    url      = "https://api.search.brave.com/res/v1/web/search"
    headers  = {
      "X-Subscription-Token" = "{{ env.BRAVE_API_KEY }}"
      Accept                 = "application/json"
    }
    query = {
      q     = "{{ args.query }}"
      count = "{{ args.count }}"
    }
  }
}
```

**`http.web-fetch.hcl`** — fetch and extract page text:
```hcl
command "web-fetch" {
  description = "Fetch a web page and return its text content"
  mode        = "read"

  param "url" { type = "string"; required = true }

  operation {
    protocol = "http"
    method   = "GET"
    url      = "{{ args.url }}"
    headers  = {
      "User-Agent" = "kb-mcp-researcher/0.1"
      Accept       = "text/html"
    }
  }
}
```

### Research Workflow

The agent follows this flow when given a topic:

1. **Search existing vault** — `kb-mcp search --query "<topic>"` to avoid duplicates
2. **Web search** — Earl `web-search` for recent papers, tools, projects
3. **Evaluate results** — read snippets, select most relevant 2-3 sources
4. **Fetch full content** — Earl `web-fetch` on selected URLs
5. **Synthesize** — write a vault entry with proper frontmatter, citing sources
6. **Write to vault** — `kb_write` with title, tags, body, sources
7. **Report** — list what was added and any topics worth following up

### Justfile Commands

```just
# Build researcher container image
build:
    docker compose build

# Interactive research session
research:
    docker compose run --rm researcher zeroclaw chat

# Research a specific topic
research-topic topic:
    docker compose run --rm researcher zeroclaw chat \
        --prompt "Research the following topic and add relevant findings to the vault: {{topic}}"

# List what the agent can see in the vault
vault-status:
    docker compose run --rm researcher kb-mcp list-sections
```

## Acceptance Criteria

- [x] Multi-stage Dockerfile builds ZeroClaw + kb-mcp + Earl from source
- [x] Container starts with read-only root, tmpfs, non-root user
- [x] Vault mounted as read-write volume at `/workspace/vault`
- [x] `kb-mcp list-sections` works inside the container
- [x] `kb-mcp search --query "memory"` returns vault results
- [x] DuckDuckGo web search works (switched from Brave — no API key needed)
- [x] Agent can create a new vault entry via `kb_write`
- [x] New vault entry appears on host filesystem (volume mount working)
- [x] IDENTITY.md and SOUL.md mounted read-only
- [x] `just agent-research` starts an interactive session
- [x] `just agent-research-topic "HNSW vector search"` produces a vault draft
- [x] Drafts pipeline: agent writes to `vault/drafts/`, human reviews + promotes

## Implementation Phases

### Phase 1: Dockerfile + Container Skeleton

1. Create `Dockerfile` with multi-stage build (ZeroClaw, kb-mcp, Earl)
2. Create `docker-compose.yml` with volume mounts and security model
3. Create `agents/researcher/workspace/IDENTITY.md`
4. Create `agents/researcher/workspace/SOUL.md`
5. Create `agents/researcher/workspace/collections.ron`
6. Verify: `docker compose build` succeeds, container starts, `kb-mcp list-sections` works

### Phase 2: Earl Templates + Research Capability

1. Create `templates/http.web-search.hcl` (Brave Search)
2. Create `templates/http.web-fetch.hcl`
3. Configure Earl inside the container (templates + secrets)
4. Verify: `earl call http.web-search --query "AI agent memory"` returns results

### Phase 3: Agent Workflow + Testing

1. Wire kb-mcp as MCP server inside the container (`.mcp.json`)
2. Add justfile commands (`build`, `research`, `research-topic`, `vault-status`)
3. Test full workflow: search vault → web search → fetch → synthesize → kb_write
4. Verify: new vault entry appears on host, properly formatted with frontmatter

### Phase 4: Documentation

1. Add `agents/researcher/README.md` with setup and usage
2. Update project `README.md` with researcher agent section
3. Add a vault entry about the researcher agent itself (dogfooding!)
4. Update `docs/GOALS.md` if needed

## Alternative Approaches Considered

| Approach | Why rejected |
|----------|-------------|
| IronClaw container | WASM sandbox not needed for a research agent. ZeroClaw is proven. Can migrate later. (See brainstorm) |
| OpenFang orchestration | Too much overhead for a single agent. (See brainstorm) |
| Git from inside container | Adds SSH key management complexity. Human review + host commit is simpler. (See brainstorm) |
| Network MCP (no volume mount) | Requires HTTP transport that doesn't exist yet (Phase 4 roadmap). (See brainstorm) |
| Heartbeat scheduling | Start manual, add scheduling once quality is proven. (See brainstorm) |

## Future Enhancements

(From brainstorm — each separate from initial build)

1. **Gap analyzer agent** — reads vault, identifies thin/missing topics
2. **Knowledge keeper agent** — combines researcher + gap analysis + pruning
3. **Heartbeat scheduling** — weekly research sweeps on configured topics
4. **Network MCP** — when HTTP transport ships, drop the volume mount

## Dependencies & Prerequisites

- Docker + Docker Compose
- Brave Search API key (for web research)
- Host Ollama (for dev mode LLM) or Anthropic/OpenAI API key (for prod)
- kb-mcp source at github.com/ttdonovan/kb-mcp (built in Dockerfile)

## Sources & References

### Origin

- **Brainstorm:** [docs/brainstorms/2026-03-19-container-researcher-agent-brainstorm.md](../brainstorms/2026-03-19-container-researcher-agent-brainstorm.md) — ZeroClaw runtime, volume mount, web research via Earl, manual trigger

### Reference Architecture

- Prior production agent project — proven container pattern (Dockerfile, compose, IDENTITY/SOUL, Earl templates)
- IronClaw runtime — future migration target if WASM sandbox is needed
- Knowledge Keeper pattern — `vault/patterns/knowledge-keeper.md`

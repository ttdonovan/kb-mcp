# Brainstorm: Container Researcher Agent

**Date:** 2026-03-19
**Status:** Design complete

## What We're Building

A containerized research agent that uses kb-mcp to discover new content
about AI agent memory (papers, tools, projects) and curate it into the
vault. Runs in a single ZeroClaw container with web search capabilities
via Earl. Manually triggered to start, with heartbeat automation as a
future enhancement.

This agent is the first consumer of kb-mcp beyond Claude Code sessions —
it dogfoods the tool in a containerized, autonomous context. The vault
it curates is the same vault that documents how to build systems like
itself.

## Why This Approach

**ZeroClaw over IronClaw** — ZeroClaw is battle-tested in the
a prior production agent project. The Dockerfile, container security
model (read-only root, tmpfs, named volumes), and Earl integration are
proven patterns. IronClaw's WASM sandbox adds security value but isn't
needed for a research agent that only reads the web and writes markdown.
Can migrate to IronClaw later if the security model matters.

**Volume mount over network MCP** — the vault is a directory of markdown
files. Mounting it as a read-write volume is the simplest path: the agent
writes files, you review + git commit from the host. Network MCP (Phase 4
roadmap) would enable cleaner isolation but requires HTTP transport that
doesn't exist yet.

**Manual trigger over heartbeat** — start simple. You trigger research
sessions when you want them, stay in the loop for quality. Add
HEARTBEAT.md-based scheduling once the agent's output quality is proven.

## Key Decisions

| Decision | Choice | Alternatives Considered |
|----------|--------|------------------------|
| Runtime | ZeroClaw (single container) | IronClaw (WASM sandbox), OpenFang (full orchestration — too much overhead) |
| Vault access | Volume mount (read-write) | Git from inside container, kb_write over network MCP (Phase 4) |
| Research tools | Web search + page fetch via Earl | Search only (no fetch), manual URL seeding |
| Trigger model | Manual (`just research` / interactive chat) | Heartbeat schedule, event-driven |
| kb-mcp in container | Build from source in multi-stage Dockerfile | Copy pre-built binary from host |
| LLM provider | Configurable (Ollama for dev, Anthropic/OpenAI for prod) | Hardcoded provider |

## Scope: Initial Build

### Container Setup

Multi-stage Dockerfile following a proven production pattern:
- **Stage 1:** Build ZeroClaw from source (GitHub)
- **Stage 2:** Build kb-mcp from source (GitHub)
- **Stage 3:** Build Earl from source (minimal features)
- **Stage 4:** Runtime (Debian slim, both binaries + Earl)

Container security model (proven production model):
- Read-only root filesystem
- Non-root user (uid 1001)
- Named volume for workspace (writable)
- Vault mounted as read-write volume
- tmpfs for /tmp and runtime config
- Localhost-only ports
- Host Ollama access via host-gateway

### Agent Identity

```
agents/researcher/
├── config/
│   ├── config.toml.ollama.example
│   └── config.toml.anthropic.example
├── workspace/
│   ├── IDENTITY.md          # Research domain, available tools, boundaries
│   ├── SOUL.md              # Decision principles, quality standards
│   └── collections.ron      # kb-mcp config pointing to mounted vault
└── .mcp.json                # Registers kb-mcp as MCP server
```

**IDENTITY.md** — defines the agent as a knowledge researcher specializing
in AI agent memory. Lists available tools (kb-mcp search/write, Earl web
search/fetch). Sets boundaries: only writes to vault, never modifies
existing files (human reviews changes), sources every claim.

**SOUL.md** — quality principles: prefer primary sources (papers, official
docs) over blog posts, always include source URLs in frontmatter, don't
duplicate existing vault content (search first), flag uncertainty rather
than fabricating.

### kb-mcp Integration

kb-mcp registered as an MCP server inside the container:
```json
{
  "mcpServers": {
    "kb": {
      "command": "kb-mcp",
      "args": []
    }
  }
}
```

`collections.ron` inside the workspace points to the mounted vault:
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

### Earl Templates for Research

Minimal set of Earl templates for web research:
- `http.web-search` — search via Brave Search API or similar
- `http.web-fetch` — fetch and extract text from a URL (via defuddle or similar)

Secrets (API keys) managed via Earl's credential injection — never in
container env vars or mounted files.

### Research Workflow

```
1. Agent receives topic (manual prompt or future heartbeat)
2. Search existing vault via kb-mcp (avoid duplicating content)
3. Web search for recent papers, tools, projects
4. Fetch and read promising results
5. Synthesize into a vault entry with proper frontmatter + sources
6. Write via kb_write to the writable vault collection
7. Report what was added
```

### Docker Compose

```yaml
services:
  researcher:
    build: .
    profiles: [dev]
    volumes:
      - researcher-workspace:/workspace
      - ./vault:/workspace/vault          # vault mounted read-write
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

volumes:
  researcher-workspace:
```

### Justfile Commands

```just
# Build researcher container
build:
    docker compose build

# Start interactive research session
research:
    docker compose run --rm researcher zeroclaw chat

# Start with specific topic
research-topic topic:
    docker compose run --rm researcher zeroclaw chat --prompt "Research: {{topic}}"
```

## Future Enhancements

Ordered by value, each is separate from the initial build.

### Gap Analyzer Agent

A second agent (or mode) that reads the existing vault, identifies thin
or missing topics, and either writes new content or creates a TODO list
for the researcher. Inward-facing complement to the outward-facing
researcher.

### Knowledge Keeper Agent

Combines researcher + gap analyzer into the full Knowledge Keeper pattern
documented in `vault/patterns/knowledge-keeper.md`. Sweeps sessions,
scores knowledge by usefulness, prunes stale entries. The most autonomous
form of vault curation.

### Heartbeat Scheduling

Add HEARTBEAT.md to the researcher agent. Runs a weekly research sweep
on configured topics. Reports findings via a digest file or notification.

### Network MCP (Phase 4)

When kb-mcp gains HTTP transport, the vault doesn't need to be volume-
mounted. The agent talks to a long-lived kb-mcp daemon over HTTP. Cleaner
isolation — the container never touches the filesystem directly.

## Open Questions

*None — all resolved during brainstorm.*

## Resolved Questions

| Question | Resolution |
|----------|------------|
| IronClaw or ZeroClaw? | ZeroClaw — proven container pattern, simpler. IronClaw later if needed. |
| How does the vault get written? | Volume mount. Network MCP is a future enhancement (Phase 4). |
| What research tools? | Web search + fetch via Earl templates. |
| How is kb-mcp delivered? | Built from source in multi-stage Dockerfile alongside ZeroClaw. |
| Trigger model? | Manual only to start. Heartbeat scheduling as future enhancement. |
| Agent scope? | Researcher only. Gap analyzer and knowledge keeper as future agents. |

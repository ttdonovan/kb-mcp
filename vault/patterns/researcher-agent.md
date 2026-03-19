---
tags: [patterns, agents, automation, kb-mcp]
created: 2026-03-19
updated: 2026-03-19
---

# Researcher Agent Pattern

A containerized agent that autonomously discovers and curates knowledge
into a markdown vault. The agent searches the web for new content,
evaluates relevance against existing vault entries, and writes new
documents with proper frontmatter and source citations.

## Why Containerize?

Running the research agent in a container provides:

- **Isolation** — the agent can only access the vault (volume mount) and
  the web (Earl templates). No access to host filesystem, credentials,
  or other projects.
- **Reproducibility** — same container, same behavior. No "works on my
  machine" issues.
- **Security** — read-only root, non-root user, tmpfs for temp files.
  API keys via environment variables, not baked into the image.

## Architecture

The agent runs in a single ZeroClaw container with three tools:

- **kb-mcp** — searches the vault, writes new entries, avoids duplicates
- **Earl** — web search (Brave API) and page fetching
- **ZeroClaw** — agent runtime with IDENTITY.md/SOUL.md personality

The vault is volume-mounted from the host. The agent writes files
directly. A human reviews and git-commits from the host.

## Identity/Soul Separation

The agent's behavior is defined by two files:

- **IDENTITY.md** — what the agent knows and can do (tools, domain,
  boundaries). Think of it as the job description.
- **SOUL.md** — how the agent makes decisions (quality standards, source
  preferences, honesty principles). Think of it as the values statement.

Both are mounted read-only — the agent cannot modify its own identity.

## Workflow

1. Receive a topic (manual prompt or future heartbeat)
2. Search existing vault to understand what's already covered
3. Web search for recent papers, tools, projects
4. Evaluate and fetch the best 2-3 sources
5. Synthesize into a vault entry with proper frontmatter
6. Write via `kb_write` to the writable collection
7. Report what was added

## When to Use

- Vault needs to stay current on a fast-moving topic
- Manual research is the bottleneck, not review quality
- You want a structured capture pipeline (search → evaluate → write)
- You're comfortable reviewing AI-generated first drafts

## When Not to Use

- The vault is small enough to maintain manually
- Topics require deep expertise the LLM lacks
- Source quality is critical and can't be verified from metadata alone
- You don't have Docker or an LLM API key

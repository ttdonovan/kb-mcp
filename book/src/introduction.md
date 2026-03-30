# kb-mcp

**Local, Rust-native MCP server for markdown knowledge bases.**

AI agents work better when they have structured access to project knowledge.
Markdown vaults — Obsidian, plain directories, skill libraries — are where
this knowledge already lives. But agents can't browse a vault. They need
search, filtering, and token-efficient retrieval over content you've already
written.

kb-mcp bridges that gap. Point it at your markdown directories, and it
exposes them as [MCP](https://modelcontextprotocol.io/) tools that any
agent can query — no cloud services, no databases, no infrastructure.

## Two Search Modes

kb-mcp ships with two search backends, chosen at build time:

**BM25 keyword search** (default) — Fast, lightweight, zero dependencies
beyond the binary. Powered by [Tantivy](https://github.com/quickwit-oss/tantivy).
Ideal for exact keyword queries like "PostgreSQL connection pool" or "rate
limit error".

**Hybrid BM25 + vector search** (`--features hybrid`) — Adds semantic
similarity via a local ONNX embedding model (BGE-small-en-v1.5). Conceptual
queries like "how do agents share state?" now match documents titled "Shared
Memory" that keyword search alone would miss. Results are fused with
[Reciprocal Rank Fusion](usage/hybrid-search.md) — keyword precision isn't
lost, it's augmented. Still fully local, no API keys.

## Where kb-mcp Fits

kb-mcp occupies a specific niche: **local, zero-infrastructure knowledge
base server for curated markdown.** It's not agent session memory, not a
cloud service, not a code search tool.

| Need | kb-mcp | Alternatives |
|------|--------|-------------|
| Serve markdown docs to agents via MCP | Yes — BM25 or hybrid search, token-efficient briefings, write-back | obsidian-web-mcp (remote HTTP, ripgrep) |
| Agent working memory (session state, preferences) | No — use a dedicated memory system | hipocampus, hmem |
| Cloud-hosted memory service | No — fully local, single binary | mengram |
| Semantic code search | No — markdown only | mnemex |

The memory-focused projects are **complementary** — an agent could use hmem
for working memory and kb-mcp for its knowledge base. See the full
[Landscape](landscape/overview.md) survey for detailed comparisons.

## Features

- **6 MCP tools** — list, search, get, context briefing, write, reindex
- **CLI parity** — every MCP tool works as a CLI subcommand
- **RON configuration** — typed, Rust-native config with comments
- **Collection model** — multiple collections with sections, descriptions, and writable flags
- **Token-efficient** — `kb_context` returns frontmatter + summary without the full body
- **Write-back** — `kb_write` creates notes with proper frontmatter in writable collections
- **~1,700 lines of Rust** — small, auditable, single-binary

## Built With Itself

kb-mcp ships with everything you need to try it immediately. The included
`collections.example.ron` indexes the project's own documentation and an
AI agent memory vault — so cloning the repo and running `kb-mcp list-sections`
gives you a working knowledge base out of the box.

This isn't a demo afterthought. The project is both the tool and a practical
example of using it. The vault documents AI agent memory patterns. The
[Researcher Agent](agents/researcher.md) uses kb-mcp to research topics
and curate findings back into the vault. Every feature gets consumed by
the project itself — if it doesn't work well for us, it won't work well
for anyone.

Point an AI agent at this repo with kb-mcp as an MCP server and it can
search the docs, read architecture decisions, and understand how the
project works — using the very tool the project builds.

## Quick Start

```sh
# Install both binaries (CLI `kb` + MCP server `kb-mcp`)
just install

# Or install individually
cargo install --path crates/kb-cli          # installs `kb`
cargo install --path crates/kb-mcp-server   # installs `kb-mcp`

# Create config
cp collections.example.ron collections.ron
# Edit paths to point at your markdown directories

# Use as CLI
kb list-sections
kb search --query "your query"

# Use as MCP server (register in .mcp.json — binary name is still `kb-mcp`)
```

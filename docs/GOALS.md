# Goals

## Why kb-mcp Exists

AI agents work better when they have structured access to project knowledge.
Markdown vaults — Obsidian, plain directories, skill libraries — are where
this knowledge lives. But agents can't browse a vault. They need search,
filtering, and token-efficient retrieval.

kb-mcp bridges this gap: index markdown collections, expose them as MCP
tools, let agents query and contribute to the knowledge base.

## Design Principles

**Project-agnostic.** The binary knows nothing about any specific vault,
project, or directory structure. Everything comes from `collections.ron`.
One binary serves any project with markdown files.

**Configuration as data.** Section descriptions, collection paths, writable
flags — all RON config. No recompilation to change what gets indexed or
how it's described.

**Token-efficient by default.** `kb_context` exists because agents shouldn't
read 50 documents to find the 3 that matter. Frontmatter + summary first,
full content on demand.

**CLI parity.** Every MCP tool works as a CLI subcommand. Testing,
scripting, and debugging don't require an MCP client.

**Fresh reads over cached content.** `get_document` reads from disk, not
the index. Edits are visible immediately. The index is for search and
lookup — not content serving.

**Simple until proven insufficient.** Start with the simplest approach
that works. Persistent storage, vector search, and incremental reindex
come when the simple approach hits real limits.

**Dogfood everything.** This project is both the tool and a practical
example of using it. The vault documents AI agent memory. The
`collections.example.ron` indexes the project's own docs. The container
agent uses kb-mcp to research and curate the vault. Every feature we
build, we also consume — if it doesn't work well for us, it won't work
well for anyone.

## Non-Goals

These reflect the project's current focus, not permanent boundaries.
As kb-mcp matures and usage patterns emerge, any of these could become
goals in a future phase.

**Not a general-purpose search engine.** kb-mcp indexes markdown files with
YAML frontmatter. It does not index code, logs, databases, or arbitrary
file formats.

**Not a document editor.** `kb_write` creates new files. It does not edit,
rename, move, or delete existing documents. Vault management stays in the
editor (Obsidian, VS Code, etc.).

**Not a vector database.** Even with hybrid search planned, the primary
interface is BM25 keyword search. Vector similarity augments it — it doesn't
replace it.

**Not a multi-user system.** kb-mcp serves one agent session at a time
(stdio is 1:1). HTTP mode may support multiple clients but there's no
authentication, permissions, or multi-tenancy.

**Not a replacement for agent memory.** kb-mcp serves project knowledge
(docs, guides, skills). Agent memory (preferences, session state, learned
behaviors) belongs in the agent's own memory system.

**Not a web application.** No UI, no dashboard, no browser interface. CLI +
MCP is sufficient. Agents are the primary consumers.

## See Also

- [Roadmap](ROADMAP.md) — what's completed, what's next, and when
- [Architecture](ARCHITECTURE.md) — how it's built

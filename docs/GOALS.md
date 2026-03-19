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
agent (planned) uses kb-mcp to research and curate the vault. Every
feature we build, we also consume — if it doesn't work well for us, it
won't work well for anyone.

## Current State

Working standalone binary with:

- RON-based collection configuration with sections and writable flags
- 6 MCP tools: `list_sections`, `search`, `get_document`, `kb_context`,
  `kb_write`, `reindex`
- Full CLI parity for all tools
- Persistent BM25 search via memvid-core `.mv2` storage
- Incremental reindex via blake3 content hashing
- Smart markdown chunking for better search precision
- Crash-safe WAL for write durability
- Tested against ~130 documents across 27 sections

## Roadmap

### ~~Phase 1: Persistent Storage (memvid-core)~~ (Complete)

Replace the in-memory Tantivy index with memvid-core's `.mv2` persistent
storage. Each collection gets its own `.mv2` file.

**What changes:**
- Startup opens existing `.mv2` files instead of scanning + indexing
- Smart markdown chunking improves search precision on long documents
- Content hashing enables incremental reindex (only changed files)
- Crash-safe WAL protects against mid-write failures
- Deleted file detection (paths in index but missing from disk)

**When:** When startup latency matters or collection sizes exceed ~500 docs.

### Phase 2: Hybrid Search

Enable memvid-core's `vec` feature for vector similarity alongside BM25.

**What changes:**
- Local ONNX embeddings (BGE-small-en-v1.5)
- HNSW vector index stored in the `.mv2` file
- Hybrid ranking: BM25 + vector similarity via RRF fusion
- Conceptual queries ("how do agents share state?") match documents that
  don't contain the exact keywords

**When:** When keyword search consistently fails on conceptual queries.

### Phase 3: Knowledge Capture Tools

Specialized write tools for structured knowledge capture.

**What changes:**
- `kb_capture_session` — record debugging/coding sessions
- `kb_capture_fix` — record bug fixes with symptom/cause/resolution
- `kb_classify` — auto-tag unprocessed notes (type, tags, summary)

**When:** When agents are actively writing to knowledge vaults and would
benefit from structured capture templates beyond free-form `kb_write`.

### Phase 4: HTTP Daemon Mode

Add HTTP transport alongside stdio.

**What changes:**
- Long-lived server process eliminates MCP cold starts
- Multiple clients can connect simultaneously
- Becomes valuable when vector search makes startup expensive (model loading)

**When:** When cold start latency is a real problem (likely after Phase 2
adds ONNX model loading).

### Phase 5: Cross-Agent Knowledge Sharing

Multiple projects share knowledge through federated `.mv2` files.

**What changes:**
- Agents in different repos contribute to and query from shared collections
- Knowledge flows between projects without manual copying
- This is the long-term vision: knowledge sharing between agents with
  memory retention across sessions

**When:** When multiple projects are actively using kb-mcp and would benefit
from shared context.

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

**Not a vector database.** Even with Phase 2's hybrid search, the primary
interface is BM25 keyword search. Vector similarity augments it — it doesn't
replace it.

**Not a multi-user system.** kb-mcp serves one agent session at a time
(stdio is 1:1). Phase 4's HTTP mode supports multiple clients but there's
no authentication, permissions, or multi-tenancy.

**Not a replacement for agent memory.** kb-mcp serves project knowledge
(docs, guides, skills). Agent memory (preferences, session state, learned
behaviors) belongs in the agent's own memory system.

**Not a web application.** No UI, no dashboard, no browser interface. CLI +
MCP is sufficient. Agents are the primary consumers.

# Roadmap

What's been built, what's next, and when each phase makes sense.

## Completed

### Standalone MCP Server (v2)

RON-configured binary with 10 MCP tools, full CLI parity, and zero
hardcoded project-specific values. Open sourced at
[github.com/ttdonovan/kb-mcp](https://github.com/ttdonovan/kb-mcp).

### Persistent Storage (memvid-core)

Replaced in-memory Tantivy with memvid-core `.mv2` persistent files.
Incremental reindex via blake3 content hashing. Smart markdown chunking.
Crash-safe WAL.

### Containerized Researcher Agent

ZeroClaw container with kb-mcp + DuckDuckGo web search. Writes research
findings to `vault/drafts/` for human review. IDENTITY.md/SOUL.md define
agent personality and quality standards.

### Hybrid Search (kb-mcp Phase 2)

BM25 + vector search via memvid-core `vec` feature. Local ONNX embeddings
(BGE-small-en-v1.5), HNSW vector index in `.mv2` files, RRF fusion.
Opt-in via `cargo build --features hybrid`. Container supports
`just agent-build-hybrid`.

### Vault Intelligence Bundle

Three new MCP tools (`kb_digest`, `kb_query`, `kb_export`) plus transparent
auto-reindex via directory mtime checks. Brings the tool count from 6 to 9.
`kb_write` also gained optional `directory` and `filename` parameters for
hierarchical collection structures.

## Up Next

### Draft Reviewer Agent

A second container agent (or Claude Code sub-agent) that reviews drafts
for quality, formatting consistency, source verification, and proper
frontmatter — then promotes approved entries into the vault.

**Why now:** The researcher agent is producing drafts. Manual review is
the bottleneck. Automating the quality gate completes the capture pipeline.

**Scope:**
- Read drafts collection, check against vault conventions (SOUL.md standards)
- Verify sources are reachable URLs
- Ensure frontmatter has required fields (tags, created, updated, sources, target)
- Promote approved drafts to the correct vault section
- Flag issues for human attention rather than silently fixing

### Heartbeat Scheduling

Add HEARTBEAT.md to the researcher agent for automated periodic research.

**Why now:** The researcher works well manually. Scheduling is a small
addition that makes it run on autopilot for configured topics.

**Scope:**
- HEARTBEAT.md defines research topics and frequency
- ZeroClaw cron runs the research workflow on schedule
- Digest report of what was added (file or notification)

## Future

### Knowledge Capture Tools (kb-mcp Phase 3)

Specialized write tools beyond free-form `kb_write`:

- `kb_capture_session` — record debugging/coding sessions
- `kb_capture_fix` — record bug fixes with symptom/cause/resolution
- `kb_classify` — auto-tag unprocessed notes (type, tags, summary)

**When:** When agents are actively writing and would benefit from
structured capture templates.

### Gap Analyzer Agent

Reads the existing vault, identifies thin or missing topics, and feeds
the researcher agent with specific research requests. The inward-facing
complement to the outward-facing researcher.

**When:** When the vault is large enough that gaps aren't obvious from
browsing.

### Knowledge Keeper Agent

Combines researcher + reviewer + gap analyzer into the full Knowledge
Keeper pattern. Sweeps sessions, scores knowledge by usefulness, prunes
stale entries. The most autonomous form of vault curation.

**When:** When all three component agents are proven individually.

### HTTP Daemon Mode (kb-mcp Phase 4)

Add HTTP transport alongside stdio. Long-lived server eliminates MCP
cold starts and enables network-based access (no volume mount needed
for container agents).

**When:** When cold start latency is a problem (likely after hybrid
search adds ONNX model loading).

### Cross-Agent Knowledge Sharing (kb-mcp Phase 5)

Multiple projects share knowledge through federated `.mv2` files. Agents
in different repos contribute to and query from shared collections.

**When:** When multiple projects are actively using kb-mcp and would
benefit from shared context.

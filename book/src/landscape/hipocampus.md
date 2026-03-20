# hipocampus

**Source:** [github.com/kevin-hs-sohn/hipocampus](https://github.com/kevin-hs-sohn/hipocampus)
**Language:** JavaScript | **Status:** Active (26 stars)

Drop-in memory harness for AI agents with a 3-tier memory architecture
and 5-level compaction tree.

## What It Does

hipocampus manages agent session memory over time. Hot memory (~500 lines)
is always loaded. Warm memory (daily logs, knowledge base, plans) is read
on demand. Cold memory is searched via qmd hybrid search.

The key innovation is the **5-level compaction tree**: raw daily logs get
compressed into daily → weekly → monthly → root summaries via LLM-driven
summarization. A ROOT.md topic index (~100 lines) gives agents O(1)
awareness of what they know.

## Key Features

- 3-tier memory: Hot (always loaded), Warm (on-demand), Cold (search)
- 5-level compaction tree with LLM-driven summarization
- ROOT.md topic index for constant-time knowledge awareness
- Hybrid search via qmd (BM25 + vector)
- Claude Code plugin marketplace integration
- Pre-compaction hooks for automatic memory preservation
- File-based, no database

## Comparison to kb-mcp

| Aspect | hipocampus | kb-mcp |
|--------|-----------|--------|
| **Primary use** | Agent session memory | Curated knowledge base |
| **Data model** | Daily logs → compacted summaries | Markdown collections indexed for search |
| **Search** | qmd (BM25 + vector) | memvid-core (BM25 + optional vector) |
| **Write pattern** | Continuous (daily logs, auto-compaction) | On-demand (kb_write, manual curation) |
| **MCP support** | No (skill-based) | Yes (stdio transport) |

**Relationship:** Complementary. hipocampus handles what the agent
*remembers from sessions*; kb-mcp serves what the agent *looks up in
reference material*.

## Patterns Worth Adopting

- **Compaction tree** — the 5-level summarization pattern is relevant
  for kb-mcp's future Knowledge Keeper agent
- **ROOT.md topic index** — a constant-cost "what do I know?" summary
  could complement `list_sections`

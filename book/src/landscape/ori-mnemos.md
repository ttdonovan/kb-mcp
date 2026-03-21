# Ori-Mnemos

**Source:** [github.com/aayoawoyemi/Ori-Mnemos](https://github.com/aayoawoyemi/Ori-Mnemos)
**Language:** TypeScript | **Status:** Active (62 stars)

Persistent cognitive memory system for AI agents — knowledge graph with
learning layers, identity resources, and adaptive retrieval.

## What It Does

Ori-Mnemos treats agent memory as a *learning problem*, not a lookup
problem. It builds a knowledge graph from markdown files (wiki-links +
learned co-occurrence edges), runs 4-signal retrieval (semantic + BM25 +
PageRank + warmth), and continuously improves via three learning layers
that reshape the graph with every interaction.

Agents get persistent identity (goals, methodology, reminders) and a
memory system that decays, reinforces, and prunes like biological memory.
All local — markdown + SQLite, no cloud dependencies.

## Key Features

- 4-signal RRF fusion: semantic embeddings, BM25, Personalized PageRank,
  associative warmth
- 3 learning layers: Q-value reranking, co-occurrence edge learning
  (Hebbian/NPMI), stage meta-learning (LinUCB)
- Knowledge graph: wiki-links + learned co-occurrence edges with
  homeostasis normalization
- 3 memory zones: identity (slow decay), knowledge (1x), operations
  (fast decay)
- Agent identity resources: personality, goals, methodology, daily
  context, reminders
- 16 MCP tools + 5 identity resources + 16 CLI commands
- Local embeddings: all-MiniLM-L6-v2 via Hugging Face transformers
- Storage: markdown files + SQLite (indexes and learning state)
- 579+ tests (vitest)

## Notable Tools

| Tool | Purpose |
|------|---------|
| `ori_orient` | Daily briefing — status, reminders, goals, vault health |
| `ori_query_ranked` | Full retrieval with Q-value reranking + stage meta-learning |
| `ori_explore` | Recursive graph exploration with sub-question decomposition |
| `ori_warmth` | Associative field showing resonant notes in context |
| `ori_promote` | Graduate inbox notes to typed notes with classification |
| `ori_query_fading` | Low-vitality candidates for archival |

## Comparison to kb-mcp

| Aspect | Ori-Mnemos | kb-mcp |
|--------|-----------|--------|
| **Primary use** | Agentic memory with learning | Curated knowledge base retrieval |
| **Language** | TypeScript | Rust |
| **Storage** | Markdown + SQLite | Markdown + memvid-core .mv2 |
| **Search** | 4-signal RRF (semantic + BM25 + PageRank + warmth) | BM25 + optional vector (memvid-core) |
| **Learning** | 3 layers (Q-value, co-occurrence, stage meta) | None — static index |
| **Graph** | Wiki-links + learned edges + PageRank | Section hierarchy only |
| **Identity** | Goals, methodology, reminders, daily context | Not supported |
| **Decay/vitality** | 3 memory zones with configurable decay rates | None |
| **Tools** | 16 MCP + 5 resources | 9 MCP tools |
| **CLI parity** | Yes (dual-mode) | Yes (dual-mode) |
| **Auto-reindex** | Incremental embedding updates | Directory mtime detection |
| **Recall@5** | 90% (HotpotQA multi-hop) | Baseline BM25 |
| **Latency** | ~120ms (full intelligence) | Sub-100ms (BM25) |

**Relationship:** Ori is a *superset* in ambition — it does everything
kb-mcp does (markdown indexing, search, MCP tools) plus graph-based
learning, identity management, and adaptive retrieval. kb-mcp is
simpler, faster, and Rust-native. They solve overlapping but different
problems: kb-mcp is a *library* you search; Ori is a *brain* that
learns.

## Patterns Worth Adopting

- **Q-value learning on retrieval** — tracking which search results
  agents actually use (forward citations, re-recalls, dead-ends) to
  improve future ranking. Could inform a future kb-mcp relevance layer.
- **Co-occurrence edges** — notes retrieved together form stronger
  associations. Lightweight to implement on top of existing search.
- **Identity resources** — `ori://identity`, `ori://goals` etc. give
  agents persistent context. kb-mcp's `kb_context` serves a similar
  purpose but without the identity layer.
- **Memory zones with decay** — different decay rates for identity vs
  operational knowledge. Relevant for kb-mcp's future Knowledge Keeper.
- **Stage meta-learning** — learning to skip expensive retrieval stages
  when they don't contribute value. Relevant if kb-mcp adds more
  retrieval signals beyond BM25.
- **Vault health diagnostics** — `ori_health` checks index freshness,
  orphan notes, dangling links. Could complement kb-mcp's `kb_digest`.

---
tags: [tools, retrieval, comparison, mcp]
created: 2026-03-18
updated: 2026-03-18
sources:
  - https://github.com/memvid/memvid
  - https://github.com/nicobailey/knowledge-base-server
  - https://github.com/mengram/mengram
  - https://github.com/basicmachines-co/basic-memory
---

# Retrieval Tool Landscape

A survey of tools and libraries for AI agent memory and knowledge retrieval.
Focused on what's available today for MCP-based agent systems.

## MCP-Native Tools

### kb-mcp

Rust binary. Indexes markdown collections into BM25 search. RON config.

- **Search:** BM25 (Tantivy)
- **Storage:** In-memory (persistent `.mv2` planned)
- **Write-back:** Yes, via `kb_write` for writable collections
- **Token efficiency:** `kb_context` for frontmatter + summary briefings
- **Best for:** Project-specific knowledge bases with markdown vaults

### Basic Memory

Python. Knowledge graph with typed observations and relations.

- **Search:** Entity + relation queries
- **Storage:** SQLite
- **Write-back:** Yes, creates observations and relations
- **Best for:** Projects needing relationship-aware queries ("what depends on X?")

### knowledge-base-server

TypeScript. MCP server with classification pipeline and session capture.

- **Search:** Full-text
- **Storage:** File-based
- **Write-back:** Yes, with auto-classification and frontmatter generation
- **Best for:** Teams wanting automated knowledge curation

## Libraries (Not MCP, But Relevant)

### memvid-core

Rust crate. Single-file `.mv2` persistent storage with BM25 + HNSW vector search.

- **Search:** BM25 (Tantivy) + optional vector similarity (ONNX embeddings)
- **Storage:** Persistent `.mv2` files with crash-safe WAL
- **Smart chunking:** Markdown-aware segmentation for better search precision
- **Best for:** Building custom retrieval tools that need persistent, hybrid search

### Mengram

Three-tier cognitive memory (semantic, episodic, procedural) with
procedural evolution from failure analysis.

- **Search:** Hybrid (keyword + vector)
- **Storage:** Cloud or local
- **MCP mode:** Available
- **Best for:** Agents that need to learn from experience over time

## Choosing a Tool

| Need | Recommended |
|------|-------------|
| Search a markdown vault | kb-mcp |
| Track entity relationships | Basic Memory |
| Auto-classify incoming knowledge | knowledge-base-server |
| Build a custom retrieval backend | memvid-core |
| Three-tier cognitive memory | Mengram |

## What's Missing

The landscape is young. Notable gaps:

- **Contradiction detection** — no tool flags when new information conflicts
  with existing knowledge
- **Cross-project federation** — sharing knowledge between separate agent
  workspaces is manual
- **Retention policies** — no tool automatically archives or prunes stale
  knowledge
- **Confidence scoring** — facts are binary (present or absent), not scored
  by reliability or freshness

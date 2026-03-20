# mnemex

**Source:** [github.com/MadAppGang/mnemex](https://github.com/MadAppGang/mnemex)
**Language:** TypeScript | **Status:** Active (35 stars)

Local semantic code search for Claude Code — tree-sitter parsing,
embedding-based vector search + BM25 hybrid, stored in LanceDB.

## What It Does

mnemex indexes codebases using tree-sitter to understand code structure
(functions, classes, methods), embeds chunks via configurable providers
(OpenRouter, Ollama, custom), and serves hybrid BM25 + vector search
over MCP. It's code search, not document search.

## Key Features

- Hybrid search: BM25 + vector similarity via LanceDB
- Tree-sitter code parsing (structure-aware, not naive line splits)
- Embedding flexibility: OpenRouter (cloud), Ollama (local), custom
- Embedding model benchmarking tool with NDCG scores
- Auto-reindex on search (detects modified files)
- Symbol graph with PageRank for importance ranking
- `mnemex pack` — export codebase to a single AI-friendly file
- 4 MCP tools: search_code, index_codebase, get_status, clear_index
- Claude Code plugin, OpenCode plugin, VS Code autocomplete

## Comparison to kb-mcp

| Aspect | mnemex | kb-mcp |
|--------|--------|--------|
| **Domain** | Code search | Document/knowledge search |
| **Parsing** | Tree-sitter (code-aware chunks) | Markdown (frontmatter + heading structure) |
| **Hybrid search** | BM25 + vector (LanceDB) | BM25 + vector (memvid-core) |
| **Embeddings** | OpenRouter / Ollama / custom | Local ONNX (BGE-small-en-v1.5) |
| **Write-back** | No (read-only) | Yes (kb_write) |
| **Auto-reindex** | Yes (on search) | No (manual reindex or startup sync) |
| **Unique feature** | Symbol graph + PageRank | Token-efficient kb_context |

**Relationship:** Different domains (code vs knowledge). Both use hybrid
BM25 + vector search but with different backends and parsing strategies.

## Patterns Worth Adopting

- **Auto-reindex on search** — detect file changes at query time instead
  of requiring explicit `reindex` calls. Low overhead for small vaults.
- **Embedding model benchmarking** — a tool to evaluate search quality
  with different models and parameters.
- **Pack/export** — exporting the full knowledge base as a single
  context-friendly file for LLM ingestion.

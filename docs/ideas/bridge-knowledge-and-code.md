# Idea: Bridge Knowledge + Code in kb-mcp

**Status:** Parked -- research done, build when ready
**Date:** 2026-03-28

## The Problem

AI agents waste 88-99% of tokens reading entire source files to find one function. Existing tools (30+) solve this with code intelligence, but every tool is either knowledge OR code, never both. No tool connects "here's the function" with "here's why it was designed this way."

## The Unique Angle

Extend kb-mcp so knowledge documents can reference and contextualize source code, and vice versa. An agent asks "I need to modify the search module" and gets:

> **Code:** `src/search.rs` -- public API surface (5 functions, 2 structs), 847 lines
> **Design decision:** BM25 is primary, vector augments via RRF
> **Known constraint:** SearchEngine holds per-collection Memvid stores behind Arc<Mutex>
> **Related:** `src/index.rs` (feeds search), `src/tools/search.rs` (MCP tool surface)

Understanding alongside precision in one query.

## Research: What Exists

### TokToken (github.com/mauriziofonte/toktoken)

C binary + MCP server. Indexes codebases at the symbol level using universal-ctags. 27 MCP tools. SQLite + FTS5. 49 languages. 88-99% token savings. Beta. Requires ctags dependency.

### LSP-as-context (codeinput.com/blog/lsp-server)

Tutorial on building LSP servers in Rust via `tower-lsp-server`. LSP features (go-to-definition, hover, find-references) could feed structured code intelligence to an indexer, but nobody has built this bridge. LSP is an input channel, not a complete solution.

### Landscape (30+ tools, 5 categories)

| Category | Examples | Token Strategy |
|---|---|---|
| Knowledge Graphs | codebase-memory-mcp, CodeGraphContext | Persistent graph; query on demand; 99%+ reduction |
| AST/Tree-sitter | AFT, AiDex, Code Pathfinder | Symbols/outlines; 50-90% savings |
| Semantic Vector | Claude Context (Zilliz), grepai, Probe | Hybrid BM25+vector; relevant snippets only |
| Context Packing | Repomix, Aider repo-map | Tree-sitter compression; ~70% reduction |
| LSP Bridges | Serena, Token Saver MCP | Precise navigation; 90%+ savings |

**Notable:** codebase-memory-mcp (66 langs, 99.2% reduction), AFT (Rust, ~40 tokens/lookup), Probe (zero setup, SIMD, Rust-native), Augment Context Engine (commercial, 400K+ files).

**Critical gap:** No tool has nailed incremental real-time updates during active development. Rust-native MCP servers are rare.

## Proposed Direction

### Where it lives: Inside kb-mcp

New `kind: "code"` collection type in RON config:

```ron
(
    collections: [
        (name: "vault", path: "vault", description: "Knowledge vault", writable: true),
        (name: "source", path: "src", description: "Rust source code", kind: "code", languages: ["rust"]),
    ],
)
```

### Parsing: Tree-sitter

- Rust-native bindings, no external deps
- AST-level accuracy, incremental parsing
- Start with Rust only, expand via grammars
- Extract: function sigs, struct/enum defs, trait/impl blocks, doc comments
- No function bodies -- the outline IS the token savings

### Cross-references

- **Manual:** frontmatter `relates_to: [src/search.rs]` in knowledge docs
- **Automatic:** match extracted symbol names against knowledge doc content

### New MCP tools (3)

| Tool | Purpose |
|---|---|
| `code_outline` | Structural outline of a file (functions, types, traits). No body code. |
| `context_for_code` | Related knowledge docs + code outline for a file/symbol |
| `code_for_concept` | Relevant code locations for a knowledge topic (reverse lookup) |

### Token savings estimate

- 500-line Rust file: ~50-80 tokens of outline vs ~2000+ raw
- Plus cross-referenced knowledge: ~200-500 tokens
- Agent gets richer understanding at 15-25% of raw file cost

## Open Questions

1. Should code outlines be searchable via existing `search` tool?
2. How granular should automatic cross-references be? (file-level vs function-level)
3. Should `get_document` work for code files?
4. Feature flag (`--features code`) to keep core lightweight?

## Alternative: Don't Build

Install TokToken or codebase-memory-mcp alongside kb-mcp. Agent uses both MCP servers. Zero effort, good-enough, no differentiation.

# Landscape

A survey of related projects in the AI agent memory and knowledge
management space. Understanding what exists helps identify where kb-mcp
fits, what patterns to adopt, and where to differentiate.

## Quick Comparison

| Project | Type | Language | Search | MCP | Write | Local/Cloud |
|---------|------|----------|--------|-----|-------|-------------|
| **kb-mcp** | Knowledge base server | Rust | BM25 + vector hybrid | stdio | Yes | Local |
| [hipocampus](hipocampus.md) | Agent memory harness | JavaScript | BM25 + vector (qmd) | No | Yes | Local |
| [obsidian-web-mcp](obsidian-web-mcp.md) | Remote vault access | Python | ripgrep full-text | HTTP | Yes | Remote |
| [mengram](mengram.md) | Cloud memory service | Python + JS | Semantic (cloud) | Yes | Yes | Cloud |
| [hmem](hmem.md) | Hierarchical memory | TypeScript | Tree traversal | stdio | Yes | Local |
| [mnemex](mnemex.md) | Semantic code search | TypeScript | BM25 + vector (LanceDB) | stdio | No | Local |

## Source Code Metrics

Measured with [tokei](https://github.com/XAMPPRocky/tokei). Source-only
— excludes eval/benchmark data, web frontends, generated files, and
node_modules.

| Project | Language | Files | Code Lines | Comments | Total Lines |
|---------|----------|------:|----------:|--------:|----------:|
| **kb-mcp** | Rust | 17 | 1,696 | 36 | 2,001 |
| hipocampus | JavaScript | 3 | 730 | 96 | 954 |
| obsidian-web-mcp | Python | 18 | 1,469 | 43 | 1,895 |
| mengram | Python (core) | 48 | 22,097 | 1,007 | 25,907 |
| hmem | TypeScript | 10 | 6,569 | 577 | 7,617 |
| mnemex | TypeScript + TSX (src/) | 388 | 86,919 | 20,113 | 120,031 |

**Notes:** mengram includes cloud backend, SDKs, and integrations in its
Python source. mnemex `src/` includes the core engine, CLI, and MCP
server — the full repo (329K lines) also contains eval benchmarks (91K),
AI docs (15K), landing page, and VS Code extension.

## Documentation Metrics

Markdown files as a proxy for documentation investment.

| Project | .md Files | Lines | Doc-to-Code Ratio |
|---------|----------:|------:|------------------:|
| **kb-mcp** | 52 | 4,817 | 2.8x |
| hipocampus | 16 | 1,657 | 2.3x |
| obsidian-web-mcp | 1 | 198 | 0.1x |
| mengram | 17 | 1,871 | 0.1x |
| hmem | 11 | 2,710 | 0.4x |
| mnemex | 74 | 23,808 | 0.3x |

**Observations:**

- **kb-mcp has the highest doc-to-code ratio** (2.8x) — more lines of
  documentation than source code. This reflects the project's dual role
  as both a tool and a documented reference implementation.
- **hipocampus** also documents heavily (2.3x) — its memory architecture
  and protocol are extensively explained in markdown.
- **obsidian-web-mcp** has minimal docs (single README) despite a
  substantial codebase — the code is the documentation.
- **mnemex** has extensive docs (74 files, 24K lines) but the ratio is
  low because the codebase is so large.

## Where kb-mcp Fits

kb-mcp occupies a specific niche: **local, Rust-native, zero-infrastructure
knowledge base server for curated markdown.** It's not trying to be agent
session memory (hipocampus, hmem), a cloud service (mengram), or a code
search tool (mnemex).

The closest overlap is with obsidian-web-mcp (both serve markdown vaults
via MCP), but they differ on transport (local stdio vs remote HTTP) and
search quality (BM25-ranked vs ripgrep grep).

The memory-focused projects (hipocampus, mengram, hmem) are **complementary**
rather than competitive — they manage agent session memory, while kb-mcp
serves reference knowledge. An agent could use hmem for working memory
and kb-mcp for its knowledge base.

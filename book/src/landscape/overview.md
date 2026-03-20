# Landscape

A survey of related projects in the AI agent memory and knowledge
management space. Understanding what exists helps identify where kb-mcp
fits, what patterns to adopt, and where to differentiate.

## Quick Comparison

| Project | Type | Search | MCP | Write | Local/Cloud |
|---------|------|--------|-----|-------|-------------|
| **kb-mcp** | Knowledge base server | BM25 + vector hybrid | stdio | Yes | Local |
| [hipocampus](#hipocampus) | Agent memory harness | BM25 + vector (qmd) | No | Yes | Local |
| [obsidian-web-mcp](#obsidian-web-mcp) | Remote vault access | ripgrep full-text | HTTP | Yes | Remote |
| [mengram](#mengram) | Cloud memory service | Semantic (cloud) | Yes | Yes | Cloud |
| [hmem](#hmem) | Hierarchical memory | Tree traversal | stdio | Yes | Local |
| [mnemex](#mnemex) | Semantic code search | BM25 + vector (LanceDB) | stdio | No | Local |

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

---
tags: [memory, llm, agents, rag, vector-search, knowledge-graphs, mcp, tooling]
created: 2026-03-19
updated: 2026-03-19
status: draft
source: https://github.com/mem0ai/mem0 ; https://docs.mem0.ai/ ; https://arxiv.org/abs/2504.19413 ; https://mem0.ai/research
---

# Mem0 memory layer for LLM applications

# Mem0 (mem0ai) — memory layer for LLM applications

## What it is
Mem0 (“mem-zero”) is an open-source + hosted **memory layer** intended to give LLM apps/agents persistent, personalized context across sessions without stuffing entire chat histories into the prompt. It provides SDKs/APIs for **adding**, **searching**, **updating**, and **deleting** memories, plus optional **graph memory**.

- Repo: https://github.com/mem0ai/mem0
- Docs: https://docs.mem0.ai/
- Paper: *Mem0: Building Production-Ready AI Agents with Scalable Long-Term Memory* (Chhikara et al., 2025) https://arxiv.org/abs/2504.19413

## Core ideas (from the paper)
Mem0 is a **memory-centric architecture** that *extracts*, *consolidates*, and *retrieves* salient information from conversations.

### Two-phase pipeline: Extraction → Update
From the paper’s method description (Mem0):

1. **Extraction phase**
   - Inputs: (a) latest message pair (typically user message + assistant response), (b) a **rolling conversation summary**, and (c) **m most recent messages**.
   - Uses an LLM to extract a small set of candidate **salient “memory facts”**.
   - Summary refresh can run asynchronously so inference isn’t blocked.

2. **Update phase**
   - For each candidate fact, retrieve the **top-*s* similar** existing memories from a vector DB.
   - Use the LLM (via tool/function-calling style decision) to pick one of:
     - **ADD** (new memory)
     - **UPDATE** (augment/replace an existing memory)
     - **DELETE** (remove contradicted memory)
     - **NOOP** (nothing to do)
   - Goal: keep memory store coherent, non-redundant, and current.

### Graph variant (Mem0^g)
Mem0^g extends Mem0 with a **directed, labeled graph memory** where:
- Nodes represent **entities** (with embeddings + metadata like timestamps)
- Edges represent **relations** as triplets (source, relation, destination)

Retrieval in Mem0^g combines:
- **Entity-centric subgraph retrieval** (anchor entities from query, then expand neighborhood)
- **Semantic triplet matching** (embed query, compare against encoded triplets)

## Empirical claims (LOCOMO benchmark)
Reported highlights on LOCOMO (multi-session long conversations):

- **+26% relative improvement** in “LLM-as-a-Judge” score over OpenAI’s memory approach.
- **~91% lower p95 latency** vs. full-context (re-sending the entire ~26k-token conversation each time).
- **>90% token savings** vs. full-context.

Sources: https://mem0.ai/research and https://arxiv.org/abs/2504.19413

## Product/API model (what developers use)
Mem0 exposes a “memory CRUD + search” interface.

### Managed platform
Use `MemoryClient(api_key=...)` and call `add(...)`, `search(...)`, etc.

Source: https://docs.mem0.ai/platform/quickstart

### Open-source (self-host)
Use `Memory()` (Python) with defaults, or override components via config.

Defaults in docs include:
- LLM: OpenAI `gpt-4.1-nano-2025-04-14`
- Embeddings: `text-embedding-3-small`
- Vector store: local Qdrant at `/tmp/qdrant`
- History store: SQLite at `~/.mem0/history.db`

Source: https://docs.mem0.ai/open-source/overview and https://docs.mem0.ai/open-source/python-quickstart

### “Infer” vs “raw transcript” mode
When adding memories:
- `infer=True` (default): extract structured memories + run conflict resolution
- `infer=False`: store raw messages as-is (skips dedupe/conflict resolution)

Source: https://docs.mem0.ai/core-concepts/memory-operations/add

## Memory layering concepts
Mem0 documentation describes layers:
- **Conversation memory**: in-flight messages within a turn
- **Session memory**: short-lived facts for a task/channel
- **User memory**: long-lived, user-scoped preferences/facts
- **Org memory**: shared context across agents/teams

Source: https://docs.mem0.ai/core-concepts/memory-types

## Graph memory in OSS
Mem0 OSS can be configured with a graph backend (Neo4j, Memgraph, Neptune, Kuzu) to store edges alongside embeddings.

Docs note:
- Vector search returns ranked `results`
- Graph retrieval runs in parallel and adds extra context (e.g., `relations`) but **does not automatically reorder vector hits**.

Source: https://docs.mem0.ai/open-source/features/graph-memory

## MCP integration
Mem0 provides an MCP server exposing memory operations as tools (so agents can decide when to write/search/update memory):
`add_memory`, `search_memories`, `get_memories`, `update_memory`, `delete_memory`, etc.

Source: https://docs.mem0.ai/platform/mem0-mcp

## References
- https://github.com/mem0ai/mem0
- https://mem0.ai/research
- https://arxiv.org/abs/2504.19413
- https://docs.mem0.ai/


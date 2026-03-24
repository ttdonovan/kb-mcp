# Agentic Design Patterns

**Source:** [github.com/Mathews-Tom/Agentic-Design-Patterns](https://github.com/Mathews-Tom/Agentic-Design-Patterns)
**Type:** Reference book (21 chapters) | **Status:** Active

Comprehensive open-source book covering agentic AI patterns — memory
management, learning, MCP, RAG, multi-agent collaboration, tool use,
planning, guardrails, and more. Grounded in Google ADK, LangChain, and
LangGraph with hands-on code examples.

This page maps the book's patterns against kb-mcp to identify what
we're doing well, where gaps exist, and what belongs in future work.

## Relevant Chapters

| Chapter | Topic | Relevance to kb-mcp |
|---------|-------|-------------------|
| Ch 5 | Tool Use (Function Calling) | Direct — kb-mcp's 10 MCP tools follow this pattern |
| Ch 8 | Memory Management | Core — defines short-term vs long-term memory architecture |
| Ch 9 | Learning and Adaptation | Informs future agent memory project |
| Ch 10 | Model Context Protocol (MCP) | Direct — validates kb-mcp's MCP implementation |
| Ch 14 | Knowledge Retrieval (RAG) | Direct — kb-mcp implements the RAG pattern |

## What kb-mcp Gets Right

### MCP Implementation (Ch 10)

The book describes MCP as a "universal adapter" with client-server
architecture, tool discovery, and standardized communication. kb-mcp
follows this exactly — stdio transport, `#[rmcp::tool]` with
JsonSchema params, structured JSON output.

The book warns about wrapping legacy APIs without making them
"agent-friendly" — returning formats agents can't parse (like PDFs
instead of markdown). kb-mcp avoids this: all output is structured
JSON, tool descriptions guide agent behavior, and `kb_context`
provides token-efficient previews before full retrieval.

### RAG Pattern (Ch 14)

kb-mcp implements the core RAG pipeline the book describes:

1. **Chunking** — smart markdown chunking via memvid-core
2. **Embeddings** — optional BGE-small-en-v1.5 via hybrid feature
3. **Vector storage** — persistent .mv2 files
4. **Retrieval** — BM25 + semantic search with RRF fusion

The book's "Agentic RAG" pattern — where an agent reasons about
retrieval quality — maps to how agents use kb-mcp's progressive
disclosure chain: `kb_digest` (vault overview) then `search` (find
candidates) then `kb_context` (preview metadata) then `get_document`
(full content). Each step lets the agent decide whether to go deeper.

### Tool Use Pattern (Ch 5)

The book's tool use lifecycle matches kb-mcp exactly:

1. Tool definitions with descriptions and typed parameters
2. LLM decides which tool to call based on the task
3. Structured output (JSON) with the tool result
4. LLM processes the result and decides next steps

kb-mcp's "primitives over workflows" approach is validated — tools are
composable building blocks, not opinionated workflows. An agent can
combine `search` + `kb_query` + `get_document` in whatever order
serves the task.

## Where Gaps Exist

### No Short-Term Memory / Session State (Ch 8)

The book's primary memory pattern is the **dual memory system**:

- **Short-term** — session context, recent interactions, task state
- **Long-term** — persistent knowledge store, searchable repository

kb-mcp provides the long-term side (search, retrieval, export) but has
zero session awareness. It doesn't know what the agent searched for
previously, which documents were already retrieved, or what the agent's
current goal is. Every tool call is stateless.

The book's ADK framework solves this with `Session` (chat thread),
`State` (temporary key-value data with scoped prefixes), and
`MemoryService` (long-term searchable store). LangGraph uses
`InMemoryStore` with namespaced keys.

**Assessment:** This is not a gap in kb-mcp — it's a gap in the
*system architecture*. Session state belongs in the agent framework
(ADK, LangGraph, Claude Code), not the knowledge base. kb-mcp is the
long-term memory store; the framework provides short-term context.

### No Learning Loop (Ch 9)

The book describes agents that improve through:

- **Reinforcement learning** — rewards for good outcomes
- **Memory-based learning** — recalling past experiences
- **Self-modification** — agents editing their own behavior

kb-mcp is completely static — search results don't improve based on
which documents agents actually use. The Q-value pattern from
[Ori-Mnemos](ori-mnemos.md) maps to Chapter 9's "Memory-Based
Learning" category.

**Assessment:** Learning belongs in a future agent memory project, not
kb-mcp. See the [Ori-Mnemos analysis](ori-mnemos.md) for the pattern
evaluation.

### No Memory Type Distinction (Ch 8)

The book identifies three types of long-term memory:

- **Semantic memory** — facts and concepts (domain knowledge)
- **Episodic memory** — past experiences (successful task patterns)
- **Procedural memory** — rules and behaviors (system prompts)

kb-mcp treats all vault content as undifferentiated documents. The
section-based organization (`concepts/`, `patterns/`, `drafts/`) is a
weak form of semantic categorization, but there's no support for
episodic memory (session transcripts, successful patterns) or
procedural memory (agent instructions that evolve).

**Assessment:** kb-mcp's vault *could* map collections to memory types
(e.g., a `sessions/` collection for episodic, `prompts/` for
procedural), but the tool doesn't enforce or leverage the distinction.
This is an interesting pattern for the future agent memory project.

### No Graph-Based Retrieval (Ch 14 — GraphRAG)

The book describes GraphRAG as superior for "complex questions that
require synthesizing data from multiple sources." kb-mcp has wiki-link
parsing in `kb_health` but doesn't use links for search ranking.
[Ori-Mnemos](ori-mnemos.md) implements this with Personalized PageRank
over wiki-link + co-occurrence edges.

**Assessment:** If kb-mcp ever needs better multi-hop retrieval, the
wiki-link graph from `kb_health` could be reused as a search signal.
Low priority — BM25 + vector is sufficient for most knowledge base
queries.

## What kb-mcp Should NOT Adopt

| Pattern | Reason |
|---------|--------|
| Session/State management | Agent framework's job (ADK, LangGraph), not the knowledge base |
| Self-modification (SICA, Ch 9) | Far beyond scope — kb-mcp is a retrieval tool |
| Cloud memory services (Vertex, Ch 8) | kb-mcp is local-first by design |
| Complex learning pipelines (Ch 9) | Belongs in a separate project per the [Ori-Mnemos brainstorm](../brainstorms/archived/2026-03-21-ori-mnemos-learnings-brainstorm.md) |

## Key Insight: kb-mcp as Knowledge Retrieval, Not Memory

The book's memory management chapter (Ch 8) defines a dual architecture:

```
Agent Framework (ADK / LangGraph / Claude Code)
├── Short-term: Session context, state, recent history
└── Long-term: Persistent knowledge store ← kb-mcp serves this role
```

kb-mcp is a **knowledge base server** that agents use for long-term
knowledge retrieval — domain knowledge, reference material, documented
solutions. It is *not* an agent memory system (as stated in
[GOALS.md](../goals.md): "Not a replacement for agent memory"). The
distinction matters: agent memory includes session state, learned
preferences, and identity — things that belong in the agent framework
or a dedicated memory system.

This validates both kb-mcp's focused scope and the conclusion from the
[Ori-Mnemos brainstorm](../brainstorms/archived/2026-03-21-ori-mnemos-learnings-brainstorm.md):
agent memory (session state, learning, identity) belongs in a separate
project that could *use* kb-mcp as its knowledge retrieval layer.

## One Pattern Worth Exploring

Chapter 9 describes **"Knowledge Base Learning Agents"** that "leverage
RAG to maintain a dynamic knowledge base of problem descriptions and
proven solutions." This is exactly what kb-mcp's `docs/solutions/`
directory does via the `/ce:compound` workflow — but manually. An agent
could automate this: after solving a problem, write the solution to the
vault via `kb_write`. The researcher agent already does something
similar for external content.

This aligns with the [Roadmap's](../roadmap.md) "Knowledge Capture
Tools (Phase 3)" — specialized write tools like `kb_capture_session`
and `kb_capture_fix` that would automate structured solution capture.
The pattern stays within kb-mcp's identity (it's writing to a knowledge
base, not managing agent state) while enabling the knowledge
accumulation loop the book describes.

# mengram

**Source:** [github.com/alibaizhanov/mengram](https://github.com/alibaizhanov/mengram)
**Language:** Python + JS SDKs | **Status:** Active (112 stars)

Human-like memory for AI agents with semantic, episodic, and procedural
memory types — procedures evolve from failures.

## What It Does

mengram is a cloud-hosted memory service that gives AI agents persistent,
personalized memory across sessions. Its key differentiator is
**procedural memory** — workflows that automatically evolve when they
fail, creating an improvement loop.

## Key Features

- 3 memory types: Semantic (facts), Episodic (events), Procedural (evolving workflows)
- Cognitive Profile — persistent user profile loaded at session start
- Claude Code hooks: auto-save after responses, auto-recall on prompts
- File upload (PDF, DOCX, TXT, MD) with vision AI extraction
- Knowledge graph
- Multi-user isolation
- Import from ChatGPT / Obsidian
- Python + JavaScript SDKs, REST API
- LangChain, CrewAI, MCP integrations
- Free tier available

## Comparison to kb-mcp

| Aspect | mengram | kb-mcp |
|--------|--------|--------|
| **Hosting** | Cloud (mengram.io) | Local (your machine) |
| **Data control** | Third-party cloud | On-disk, fully private |
| **Memory model** | 3-tier cognitive (semantic/episodic/procedural) | Document collections with sections |
| **Search** | Semantic (cloud API) | BM25 + optional local vector |
| **Auto-capture** | Yes (Claude Code hooks) | No (manual or agent-driven) |
| **Dependencies** | API key + network | Zero (Rust binary) |

**Relationship:** Different trust and deployment models. mengram is
convenient (auto-save, cloud sync, SDKs) but sends your data to a
third party. kb-mcp keeps everything local and private.

## Patterns Worth Adopting

- **Procedural memory** — workflows that evolve from failure analysis.
  Relevant to the vault's Knowledge Keeper pattern.
- **Cognitive Profile** — a structured "who is the user" document.
  Claude Code's memory system does something similar.
- **Auto-save hooks** — capturing knowledge without manual intervention.
  The researcher agent's heartbeat scheduling aims at this.

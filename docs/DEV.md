# Development Tooling & Methodology

This project doubles as a reference implementation for AI-assisted
development — it's both the tool and a practical example of using it.
Every feature was built through structured AI workflows, and the
documentation captures how.

## Tools in Use

### Claude Code

[Claude Code](https://claude.com/claude-code) is Anthropic's CLI for Claude.
It's the primary development interface — all code, config, documentation,
and vault content was authored through Claude Code sessions.

Key patterns used in this project:

- **Parallel agent dispatch** — Spawning research agents to investigate
  APIs and codebase patterns simultaneously before writing code
- **MCP server integration** — kb-mcp registered in `.mcp.json` gives
  Claude Code direct access to search and query the vault during development
- **Plan mode** — Designing architecture before writing code, then
  executing with task tracking
- **Memory system** — Persistent context across sessions for project
  decisions and user preferences

### Compound Engineering

[Compound Engineering](https://github.com/EveryInc/compound-engineering-plugin)
is a Claude Code plugin that structures development into a repeating cycle
where each unit of work makes subsequent work easier.

**Philosophy:** 80% planning and review, 20% execution. Prevention over
remediation.

**Workflow cycle:**

```
/ce:brainstorm → /ce:plan → /ce:work → /ce:review → /ce:compound
```

| Command | Purpose |
|---------|---------|
| `/ce:brainstorm` | Explore requirements, approaches, and feasibility before committing |
| `/ce:plan` | Transform concepts into detailed, executable implementation strategies |
| `/ce:work` | Execute plans with feature branches, worktrees, and task tracking |
| `/ce:review` | Multi-agent code evaluation — security, architecture, performance, simplicity |
| `/ce:compound` | Capture learnings into `docs/solutions/` so future work is faster |

**How we use it in this project:**

- `/ce:brainstorm` for exploring the memvid-core integration approach,
  container agent design, and hybrid search strategy
- `/ce:plan` for translating brainstorms into phased implementation plans
  with acceptance criteria and open questions
- `/ce:work` for executing plans with incremental commits and task tracking
- Parallel research agents for investigating memvid-core's API, ZeroClaw
  config format, and ONNX model delivery

**The compounding part:** Brainstorms and plans are preserved in
`docs/brainstorms/` and `docs/plans/`. Each document captures decisions,
alternatives considered, and lessons learned — so future sessions start
with context instead of rediscovering it.

### kb-mcp (Dogfooding)

[kb-mcp](https://github.com/ttdonovan/kb-mcp) is both the product and a
development tool. During Claude Code sessions, it's registered as an MCP
server in `.mcp.json`, giving the AI direct access to search the vault.

This means Claude Code can:

- Search existing vault content before writing new docs (avoid duplicates)
- Read document metadata via `kb_context` for token-efficient scanning
- Verify that new content fits the vault structure
- Check section coverage and identify gaps

**This is the dogfooding principle** — the same tool agents use in
production is the tool we use during development. If it doesn't work well
for us, it won't work well for anyone.

## Development Workflow

### The Brainstorm → Plan → Work Loop

Every significant feature follows this cycle:

**1. Brainstorm** (`/ce:brainstorm`)

Explore what to build through collaborative dialogue. Output is a
brainstorm document in `docs/brainstorms/` capturing decisions, rejected
alternatives, and scope boundaries.

**2. Plan** (`/ce:plan`)

Transform the brainstorm into an implementation plan with:
- Phased implementation steps
- Acceptance criteria (checkboxes)
- Open questions with defaults
- API references from research

Output is a plan document in `docs/plans/`.

**3. Work** (`/ce:work`)

Execute the plan on a feature branch:
- Create tasks from plan phases
- Implement with incremental commits
- Test continuously
- Check off acceptance criteria as completed

**4. Review** (`/ce:review`)

Multi-agent code review examining security, architecture, performance,
and simplicity. Used for complex or risky changes.

**5. Compound** (`/ce:compound`)

Capture what was learned into `docs/solutions/` so the next time a
similar problem arises, the solution is already documented.

### Feature Branch Pattern

```bash
# Start from main
git checkout -b feat/feature-name

# Work with incremental commits
git commit -m "feat(scope): description"

# When done, squash merge back to main
git checkout main
git merge --squash feat/feature-name
git commit -m "feat: full description with Co-Authored-By"

# Clean up
git branch -D feat/feature-name
git push origin --delete feat/feature-name
```

### Session Patterns

**Starting a session:**

```bash
# Claude Code has kb-mcp available via .mcp.json
# Search the vault to understand current state
kb-mcp list-sections
kb-mcp search --query "whatever you're working on"
```

**Adding vault content:**

1. Search existing content for gaps
2. Draft markdown with proper frontmatter (tags, created, updated, sources)
3. Write via `kb_write` or directly to the filesystem
4. Verify with `kb-mcp search` to confirm indexing

**Research agent workflow:**

```bash
# Build the researcher container
just agent-build

# Research a topic autonomously
just agent-research-topic "topic of interest"

# Review drafts on the host
ls vault/drafts/

# Promote approved drafts to vault sections
mv vault/drafts/good-entry.md vault/concepts/
```

## Project History

This project was built in a single extended Claude Code session:

1. **v2 Standalone Crate** — Brainstormed generalizing the in-repo kb-mcp
   into a standalone project. Scaffolded in `sandbox/`, ported all 6 tools,
   added RON config, pushed to GitHub.

2. **Persistent Storage** — Replaced in-memory Tantivy with memvid-core
   `.mv2` persistent files. Added incremental reindex via blake3 content
   hashing.

3. **Containerized Researcher Agent** — Built a ZeroClaw container with
   kb-mcp + DuckDuckGo web search. Agent writes research drafts to
   `vault/drafts/` for human review.

4. **Hybrid Search** — Added opt-in BM25 + vector search via memvid-core
   `vec` feature. Local ONNX embeddings with RRF fusion.

Each phase followed the brainstorm → plan → work loop. All brainstorms
and plans are in `docs/brainstorms/` and `docs/plans/`.

## Why This Matters

This project demonstrates that a single developer with Claude Code and
structured workflows can build and maintain a complex system — Rust MCP
server, persistent search engine, containerized agent, hybrid vector
search, Obsidian vault, mdBook documentation — that would traditionally
require a team and weeks of work.

The key enablers:

1. **Structured planning** — Compound Engineering's brainstorm/plan/work
   cycle prevents the "just start coding" trap
2. **Parallel research** — Multiple agents investigate APIs, crate docs,
   and codebase patterns simultaneously
3. **MCP integration** — The knowledge base is queryable during development,
   not just at runtime
4. **Dogfooding** — Using the same tools in development that agents use
   in production catches design issues early
5. **Knowledge compounding** — Every brainstorm, plan, and solution is
   preserved so future sessions start with context

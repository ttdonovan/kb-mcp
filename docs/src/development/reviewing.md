---
description: Read before reviewing a change or acting on a review: what the code and docs reviewers check, which doc goes with which code, the report and the review log
---

# Reviewing a Change

A change is reviewed by an agent that did not write it. The `kb-mcp-dev` plugin has two reviewers, each read-only and run in a forked context, so it sees the branch and the rules but not the conversation that wrote the change:

| Change | Reviewer | Skill |
|---|---|---|
| Code, scripts, CI, the plugins' manifests and agents | `code-reviewer` | `/kb-mcp-dev:review-code [branch]` |
| Either book (`docs/src/`, `book/src/`), `README.md`, `CONTRIBUTING.md`, `CLAUDE.md`, the plugins' skills | `docs-reviewer` | `/kb-mcp-dev:review-docs [branch]` |

A branch that changes both gets both. A reviewer advises; it does not approve. This chapter is their checklist: change it here, not in the agents.

## The code reviewer

1. **The hard rules** (`CLAUDE.md`): no hardcoded paths or project values; every MCP tool has a CLI command with the same parameters and defaults; `get_document` reads from disk; `kb_write` writes only to a writable collection and never outside it; kb-core does not depend on rmcp, schemars or clap.
2. **The shared output contract:** the tool and the CLI print through the same `kb_core::format` function.
3. **The public interface** (`CONTRIBUTING.md`, Versioning): a change to a tool, the JSON output, a CLI command, a `collections.ron` field, an environment variable or the config resolution order is breaking, and its commit says so with `!`.
4. **The index:** `.mv2` stores and `.hashes` sidecars are written to a temporary file and renamed into place; a live store is never deleted or truncated; a server that only reads opens read-only. Two processes sharing a cache dir must not corrupt or lose an index.
5. **The MCP transport:** nothing but the protocol goes to stdout in `kb-mcp`; logs go to stderr through `tracing`.
6. **Failure handling:** no new `unwrap()`, `expect()` or `panic!` outside tests; an error a user can act on says what to do.
7. **Tests:** a behaviour change comes with a test that fails without it.
8. **Dependencies:** a new or updated crate passes `cargo audit` and the 7-day crate-age check; an entry added to `.cargo/audit.toml` says why the advisory does not apply.
9. **The docs:** each file changed has its docs changed with it, by "Keeping the docs true" below.
10. **The gate:** `just ci` passes, and `just plugin-validate` if a plugin changed.

## The docs reviewer

1. **Every claim is true of the code:** tool names, parameters and defaults; CLI commands and flags; config fields and defaults; recipes; paths.
2. **Planned is not presented as built:** what `book/src/roadmap.md` or `docs/project/plans/` lists as future is not described as working.
3. **The docs agree:** the change leaves no other doc saying something different, by "Keeping the docs true" below.
4. **Each page serves its reader:** usage pages for someone setting kb-mcp up, reference pages complete, development pages for contributors.
5. **Each book keeps its rules** (`docs/CLAUDE.md`, `book/CLAUDE.md`): a `description` on every chapter, the product book claiming nothing the technical book does not back.
6. **Links resolve:** `just docs check` and `just book check` pass for the books changed.

## Keeping the docs true

| A change to | Goes with |
|---|---|
| `crates/kb-mcp-server/src/tools/` | `docs/src/reference/tools.md`, the Tools table in `README.md`, the instructions in `server.rs`, the tool count and tree in `CLAUDE.md`, and `plugins/kb-mcp/skills/kb-mcp/SKILL.md` if agents use it differently |
| `crates/kb-cli/src/main.rs` | `docs/src/usage/cli.md`, the CLI section of `README.md` |
| `crates/kb-core/src/config.rs` | `docs/src/reference/ron-schema.md`, `docs/src/usage/configuration.md`, `collections.example.ron`, the Configuration section of `README.md`, the user skill |
| `crates/kb-core/src/store.rs`, `search.rs`, `index.rs` | `docs/src/development/architecture.md` |
| `justfile` | the commands in `CLAUDE.md`, the Development section of `README.md` |
| `scripts/ci/`, `.github/`, `release-plz.toml`, `.cargo/audit.toml` | `CONTRIBUTING.md` |
| `plugins/`, `.claude-plugin/` | [Claude Code Plugins](plugins.md) |
| A feature built, or a tool added, that a user would notice | `book/src/introduction.md`, and `book/src/roadmap.md` if it was planned there |
| `docs/book.toml`, `book/book.toml`, `docs/justfile`, `book/justfile`, `scripts/ci/docs-check.sh` | `docs/CLAUDE.md`, `book/CLAUDE.md`, [About this book](../README.md) |

## The report

The report opens with the branch, its base and the commit reviewed (`git rev-parse --short <branch>`), then lists findings by severity:

- **Blocker:** wrong behaviour, data loss, a security problem, a broken hard rule, or a failing gate.
- **High:** a breaking change not marked as one, a missing test for a behaviour change, or a doc now false.
- **Low:** a gap that misleads no one yet, such as a missing doc comment.
- **Optional:** a suggestion the author may take or leave.

Each finding gives its evidence (`file:line` or command output) and what would fix it. The report ends with the checks run, those that could not run, and "No findings" if there were none.

## The review log

Whoever acts on a report records the round in `.ai/reviews/<code|docs>/<branch>.md` (each `/` in the branch name becomes `-`): the commit reviewed, each finding, and what was done with it, fixed or rejected with a reason. `.ai/` is gitignored. The next round reads the log first, checks the fixes, and raises a rejected Blocker or High again if the reason does not hold.

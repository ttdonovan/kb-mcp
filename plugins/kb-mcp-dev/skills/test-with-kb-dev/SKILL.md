---
name: test-with-kb-dev
description: "Test a change to kb-mcp through a real MCP client: drive the kb-dev server (kb-mcp built from the checkout) from this session, call every tool, the error paths and automatic reindexing, and report what passed. Use after changing the server, its tools or its dependencies (such as an rmcp upgrade), or when asked to try kb-mcp in Claude Code."
---

# Test with kb-dev

The `kb-dev` server is kb-mcp built from this checkout (`docs/src/development/plugins.md`, "The kb-dev MCP server"). Calling its tools from the session tests what `crates/kb-mcp-server/tests/stdio.rs` cannot: how a real client reads each tool's schema and shows results and errors. Run both; neither replaces the other.

## Before you start

- The session was started with `just claude`, so `kb-dev` is loaded. If its tools are missing, ask the person to run `/mcp` and reconnect `plugin:kb-mcp-dev:kb-dev`.
- The server runs the code it was built from. After a code change, the person reconnects `kb-dev` in `/mcp`, which rebuilds it; you cannot. A first build can outlast the session's wait: reconnect again once it finishes.
- In Claude Code the tools are deferred: load them all first with ToolSearch, `select:mcp__plugin_kb-mcp-dev_kb-dev__search,...` with every tool name.
- It serves `collections.ron` at the root, or `collections.example.ron` without one. The example has no writable collection, so `kb_write` can only be tested failing; testing a write needs a `collections.ron` with a writable collection (gitignored, such as one under `.ai/`), then a reconnect.

## Steps

1. **Every tool, with every kind of parameter.** Call each tool once, using the parameters that are not plain strings: numbers (`max_results`, `max_documents`, `stale_days`, `min_words`), the boolean `has_sources`, a date for `created_after`, the nullable filters `collection` and `scope`, and `get_document` by title as well as by path. A schema the client misreads fails here, not in the stdio test.
2. **Error paths.** Each must come back as a tool error with a message that says what to do, never a crash or an empty success: `get_document` on a missing path, `kb_query` with `created_after: "bad"`, `kb_write` to a read-only collection, `kb_health` on an unknown collection.
3. **Automatic reindexing.** Write a probe note with a nonsense marker word (such as `zqxprobe`), so a search hits nothing else, then search for it:
   - Wait more than 5 seconds after the last tool call before searching (`sleep 6`): the server checks for changes at most once per 5 seconds, so a call sooner sees the old index.
   - Test a note at the collection's top level and one in a subfolder; staleness is judged from the top folder's modification time, so `stat` the folders to see why a result came out as it did.
   - Test an edit made in place (`printf ... >> file`) separately from one that replaces the file (`sed -i`, many editors): only the second changes the folder's time.
   - Then `reindex` and search again: that always finds it.
4. **Frontmatter.** `kb_context` on a document with frontmatter: an empty `"frontmatter": {}` means its YAML did not parse, and kb-mcp dropped it without a warning.
5. **Clean up.** Delete every probe note, check `git status` is clean, and `reindex` so the index forgets them.
6. **Report** to the person in a table: each check, pass or fail, and for a failure the cause with its `file:line`. A failure that the change did not cause is a finding for an issue, not for this branch.

## Comparing two builds

For an upgrade that should change nothing (a dependency, a refactor), also compare the old build with the new one over stdio: build the base branch in a `git worktree` under the scratchpad with its own `--target-dir`, send the same `initialize`, `tools/list` and `tools/call` messages to both binaries with `KB_MCP_CONFIG=collections.example.ron` and separate `KB_MCP_CACHE_DIR`s, parse each text result as JSON, and diff. Identical output is the evidence; remove the worktree afterwards.

## Gotchas

- The server's index is in `.ai/kb-dev/index/`; deleting it is safe and forces a full rebuild on the next start.
- Hybrid search is not covered: `kb-dev` builds without the `hybrid` feature.
- Probe notes in `vault/` show up in `git status`; never leave one behind.

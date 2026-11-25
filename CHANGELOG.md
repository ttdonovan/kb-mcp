# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.2](https://github.com/ttdonovan/kb-mcp/compare/v0.3.1...v0.3.2) - 2026-11-25

### Added

- add an exclude list of globs to a collection, for files and directories not to index, such as an mdBook's SUMMARY.md; kb_write refuses a file it would exclude.

### Fixed

- keep kb_write inside its collection: refuse a filename that is not a plain name and a directory that climbs out or follows a symlink out of the collection, before creating anything, and create the file exclusively so it never follows a symlink or overwrites a note.
- keep documents that share a path or title across collections apart: search reports each hit under its own collection, and get_document and kb_context take a collection and report an ambiguous match instead of returning the first.

## [0.3.1](https://github.com/ttdonovan/kb-mcp/compare/v0.3.0...v0.3.1) - 2026-11-20

### Added

- Claude Code plugins, from the marketplace in this repository (`claude plugin marketplace add ttdonovan/kb-mcp`): `kb-mcp`, a skill for using kb-mcp's tools and setting it up for a project, and `kb-mcp-dev`, for changing kb-mcp, with a `kb-dev` MCP server built from the checkout, code and docs reviewers, and skills for adding a tool, testing through `kb-dev`, working in a stack and releasing.
- the docs are now two books: the [technical book](https://ttdonovan.github.io/kb-mcp/) (install, configure, use, every tool and config field, development) and the [product book](https://ttdonovan.github.io/kb-mcp/product/) (goals, roadmap, landscape). Both publish `llms.txt`, `llms-full.txt` and each chapter as Markdown for agents.

### Changed

- `kb_write` and `kb write` keep a `filename` that already ends in `.md` in any case, such as `Note.MD`, instead of appending `.md`.

### Fixed

- report frontmatter that is not valid YAML in kb_health, and log it, instead of silently dropping its fields.
- *(deps)* upgrade rmcp to 3.10, which clears RUSTSEC-2026-0462; Content is now ContentBlock and ServerInfo is ServerConfig.

### Other

- make kb-mcp-server pass the strict lints: tools release the index lock before building their reply, and kb_write's frontmatter is written without format! appends.
- make kb-core pass the strict lints: no unwraps, indexing or unchecked arithmetic, a poisoned search lock recovers, and the search and health functions are split.
- add the strict clippy lints at warn, with clippy.toml allowing panics in tests, and apply clippy's automatic fixes.
- drive kb-mcp over stdio to check the handshake, the tool list and tool output survive an rmcp upgrade.
- *(deps)* update crates within their semver ranges, rmcp to 1.8, holding back any published less than 7 days ago.
- make kb-cli pass the strict lints: commands return their error to main, which exits with status 1, instead of calling process::exit, and kb-cli drops tokio.

## [0.3.0](https://github.com/ttdonovan/kb-mcp/releases/tag/v0.3.0) - 2026-11-20

### Added

- add --version to kb and kb-mcp.
- add opt-in index_names: Name, so a moved config reuses its index.
- add KB_MCP_CACHE_DIR, and resolve a relative cache_dir against the config file.
- upgrade rmcp to 1.5, add memvid-core hybrid dep, fix tool_handler router binding

### Fixed

- *(deps)* update crates with advisories, and record why the rest do not apply.
- auto-reindex a collection when another server replaced its .mv2.
- build .mv2 stores in a temp file and rename into place, open read-only.
- surface root-level docs via "(root)" section; make explicit reindex a full rebuild

### Other

- release with release-plz in git-only mode, build and attest binaries for each release, and gate PRs on fmt, clippy, tests, audit and crate age.
- apply cargo fmt to the workspace, so CI can check formatting.
- split into cargo workspace (kb-core, kb-cli, kb-mcp-server)

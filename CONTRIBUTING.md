# Contributing

## Prerequisites

- Rust, at the version pinned in `rust-toolchain.toml`
- [just](https://github.com/casey/just)
- [shellcheck](https://www.shellcheck.net/), [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit), `curl` and `jq`, for `just ci`
- [mdBook](https://rust-lang.github.io/mdBook/) 0.5, [mdbook-mermaid](https://github.com/badboy/mdbook-mermaid) 0.17 and [mdbook-llms](https://github.com/ttdonovan/mdbook-llms) 0.4, for the books: `just docs setup` installs them

## Workflow

Run `just` to list recipes. Before opening a pull request, run `just ci`: the same gate as CI (`scripts/ci/check.sh`), which checks formatting, runs clippy with warnings as errors on both the default and `hybrid` builds, runs the tests and shellcheck, checks advisories with `cargo audit`, and fails if any dependency in `Cargo.lock` is less than 7 days old (`scripts/ci/crate-age.sh`). Run `just docs check` or `just book check` if a book changed (see `docs/CLAUDE.md` and `book/CLAUDE.md`), and `just plugin-validate` if a plugin changed.

With Claude Code, `just claude` loads the `kb-mcp-dev` plugin: `/kb-mcp-dev:review-code` and `/kb-mcp-dev:review-docs` hand a branch to a read-only reviewer (see `docs/src/development/reviewing.md`).

`cargo audit` skips the advisories listed in `.cargo/audit.toml`, each with the reason it does not apply. Remove an entry once its fix lands.

See `CLAUDE.md` for the architecture, the hard rules (every MCP tool has a CLI equivalent, `kb-core` has no transport dependencies) and how to add a tool.

## Code style

Clippy enforces most of it, through `[workspace.lints]` in `Cargo.toml`, which every crate opts into: `pedantic` and `nursery` are denied, and so are panics outside tests. That means no `unwrap`, `expect`, `panic!`, `todo!`, `unreachable!`, indexing or slicing, unchecked arithmetic, `as` casts, or `process::exit`. `clippy.toml` allows `unwrap`, `expect`, indexing and `panic!` in `#[test]` code.

- Return `Result` or `Option` and handle every case. Use `get` instead of indexing, `checked_*` or `saturating_*` arithmetic, and `TryFrom` instead of `as`.
- Fix the code rather than silencing a lint, for example by splitting a function that is too long. Where a fix would make the code worse, or the lint is a false positive or fires on generated code, use `#[expect(lint, reason = "...")]` on that one item, never a bare `#[allow]`. An integration test's helpers sit outside `#[test]`, so a file in `tests/` states its own `#![expect(...)]`.

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/): prefix each message with its type, write the rest as a sentence, and end it with a full stop.

- `feat:` a new capability (a tool, a CLI command, a config field)
- `fix:` a bug fix
- `docs:`, `refactor:`, `test:`, `build:`, `ci:`, `style:`, `chore:` for the rest

Mark a breaking change with `!` (`feat!:`, `fix!:`) and say what breaks in the body.

## Versioning

The project follows [Semantic Versioning](https://semver.org/), with one version for all three crates. Its public interface is what a user, an MCP client or a config file depends on, so a change to any of these is breaking:

- MCP tools: names, parameters and their defaults
- the JSON output shared by the tools and the CLI (`kb_core::format`)
- the `kb` CLI: commands, flags and their defaults
- `collections.ron`: fields, their meaning and their defaults
- environment variables (`KB_MCP_CONFIG`, `KB_MCP_CACHE_DIR`) and the config resolution order

A change to the index files in `cache_dir` is not breaking if kb-mcp rebuilds them on its own. Adding an optional field, parameter, tool or command is not breaking.

While the version is `0.x`, a breaking change bumps the minor version (`0.4.0` to `0.5.0`) and anything else bumps the patch version.

## Releases

Releases are made by [release-plz](https://release-plz.dev/) from the commit messages. The crates are not published to crates.io (`publish = false`); release-plz runs in git-only mode, reading the current version from the `vX.Y.Z` tags. All three crates share one version (`[workspace.package]` in `Cargo.toml`), one tag and one `CHANGELOG.md` (see `release-plz.toml`).

On each push to `main`, release-plz opens or updates a release PR that bumps the version and updates `CHANGELOG.md`. Review both in that PR; merging it tags `vX.Y.Z` and creates a GitHub Release. The `Release binaries` workflow then builds `kb` and `kb-mcp` (default features) for Linux and macOS, attests them, and attaches them with a `SHA256SUMS` file to the release. To build them for an existing tag (or rebuild them), run it by hand from the Actions tab with the tag.

Install a release from its tag:

```sh
cargo install --locked --git https://github.com/ttdonovan/kb-mcp --tag vX.Y.Z kb-cli kb-mcp-server
```

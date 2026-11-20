---
description: Read to install kb and kb-mcp: prebuilt release binaries and how to verify them, building a release from its tag, and building from a clone with or without hybrid search
---

# Installation

kb-mcp is two binaries: `kb`, the CLI, and `kb-mcp`, the MCP server. Both must be on your `PATH`. kb-mcp is not published to crates.io.

## Prebuilt binaries

Each [GitHub Release](https://github.com/ttdonovan/kb-mcp/releases) has `kb` and `kb-mcp` for Linux (x86_64 and arm64, static musl) and macOS (arm64 and x86_64), with a `SHA256SUMS` file and build provenance. They are built with the default features, so BM25 search only.

```sh
TAG=vX.Y.Z TARGET=aarch64-apple-darwin
curl -fsSLO "https://github.com/ttdonovan/kb-mcp/releases/download/$TAG/kb-mcp-$TAG-$TARGET.tar.gz"
gh attestation verify "kb-mcp-$TAG-$TARGET.tar.gz" --repo ttdonovan/kb-mcp
tar -xzf "kb-mcp-$TAG-$TARGET.tar.gz"   # then put kb and kb-mcp on your PATH
```

The other targets are `x86_64-apple-darwin`, `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`. Check `kb-mcp --version` after installing.

## From a release tag

With Rust installed, build a release from its tag:

```sh
cargo install --locked --git https://github.com/ttdonovan/kb-mcp --tag vX.Y.Z kb-cli kb-mcp-server
```

## From a clone

```sh
just install                                  # both binaries
cargo install --path crates/kb-cli            # kb only
cargo install --path crates/kb-mcp-server     # kb-mcp only
```

For semantic search, add `--features hybrid` to each `cargo install`; it also needs a local embedding model, see [Hybrid Search](hybrid-search.md).

## Next

Write a `collections.ron` ([Configuration](configuration.md)), then register the server with your agent ([MCP Server](mcp-server.md)).

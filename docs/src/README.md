---
description: Start here. What kb-mcp is, how to read this book as an agent or a person, what each chapter covers, and how to build the book
---

# About this book

kb-mcp is an MCP server and CLI for markdown knowledge bases. It indexes folders of markdown ("collections", defined in `collections.ron`) into a BM25 search index, or BM25 plus vectors with the `hybrid` feature, and serves them to agents as MCP tools. The `kb` CLI has a command for every tool, with the same JSON output.

This is the technical book: how to install, configure and use kb-mcp, every tool and config field, and how to work on it. It is published at <https://ttdonovan.github.io/kb-mcp/>, built from `main`, so it can describe changes not yet released; the docs for a release are in that tag's `docs/src/` (`book/src/` for v0.3.0 and earlier). Why kb-mcp exists and how it compares with other projects is [the product book](https://ttdonovan.github.io/kb-mcp/product/).

## For agents

If you landed on an HTML page, read the plain text instead:

- [`llms.txt`](llms.txt) lists every chapter with a line on when to read it and its size. Start there.
- Then read only the chapters the task needs. Each is Markdown beside its HTML page: `usage/configuration.md` beside `usage/configuration.html`.
- [`llms-full.txt`](llms-full.txt) is the whole book in one file.

In a checkout, `just docs` writes the same files to `target/docs/html/`, matching the code you have.

## For people

| If you want to | Read |
|---|---|
| Install `kb` and `kb-mcp` | [Installation](usage/installation.md) |
| Point kb-mcp at your markdown | [Configuration](usage/configuration.md) |
| Use it from an agent | [MCP Server](usage/mcp-server.md), then [Tools](reference/tools.md) |
| Use it from a shell or a script | [CLI](usage/cli.md) |
| Find notes by meaning, not only keywords | [Hybrid Search](usage/hybrid-search.md) |
| Look up a config field | [RON Schema](reference/ron-schema.md) |
| Understand how it is built | [Architecture](development/architecture.md) |
| Change it | [Adding Tools](development/adding-tools.md), [Claude Code Plugins](development/plugins.md), [Reviewing a Change](development/reviewing.md) |

## Building it

The book is built with [mdBook](https://rust-lang.github.io/mdBook/), [mdbook-mermaid](https://github.com/badboy/mdbook-mermaid) for the diagrams, and [mdbook-llms](https://github.com/ttdonovan/mdbook-llms) for the files agents read. From the repository root:

```sh
just docs setup   # install the tools, once
just docs         # build into target/docs/html/
just docs serve   # preview with live reload
just docs check   # fail on a warning, an empty chapter, or a link that leads nowhere
```

How to write a chapter is [Writing the Docs](development/writing-docs.md).

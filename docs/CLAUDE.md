# docs/

The technical book, built for agents first: `llms.txt` (an index with each chapter's description and size), `llms-full.txt`, and each chapter's Markdown beside the HTML. Readers choose chapters from that index, so a chapter's `description` decides whether it gets read. The product book in `book/` argues for kb-mcp and says nothing this book does not back (see `book/CLAUDE.md`).

## Where to edit

- Chapters live in `docs/src/`: using kb-mcp in `usage/`, lookups in `reference/`, working on it in `development/`.
- A chapter that is only frontmatter plus an mdBook `#include` of a repository file (the researcher agent's `agents/researcher/README.md`; this file, as Writing the Docs) is written in that file: edit the file, never the wrapper or anything in `target/`. An include path is relative to the chapter. Never write the include directive itself, with its double braces, in a file the book includes: mdBook runs it.
- `docs/project/` holds the project's working records: brainstorms, plans, ideas and solutions. They are in neither book. A chapter that cites one links to it on GitHub (`https://github.com/ttdonovan/kb-mcp/blob/main/docs/project/...`), since a link outside `src/` leads nowhere on the site.

## Writing a chapter

- Start with frontmatter: `description: "..."` in one line, in double quotes (unquoted, a `: ` inside makes it invalid YAML, which kb-mcp and Obsidian drop; `just docs check` fails on it), saying when to read the chapter in the reader's terms. It becomes the chapter's line in `llms.txt`.
- Add it to `src/SUMMARY.md` and, if a reader would look for it, to the table in `src/README.md`; an unlisted file is not in the book.
- One physical line per paragraph, list item and table row; never hard-wrap prose in a new chapter, and keep an existing chapter's wrapping when editing it. No em dashes; use a colon, a comma or a hyphen.
- Plain and exact: name the tool, parameter, field, flag or path. Say what is not covered rather than implying it is. State a fact once and link to the chapter that owns it.
- Links are relative paths to other chapters, never into `target/`. A link into the product book is its published URL, `https://ttdonovan.github.io/kb-mcp/product/<chapter>.html`.
- Mermaid diagrams go in fenced `mermaid` blocks; they render in the HTML and stay as source in the llms output.
- A change to a tool, a CLI command, a config field or a recipe updates its chapter in the same change: see "Keeping the docs true" in `src/development/reviewing.md`.

## Checking

Run `just docs check` from the repository root after any change here: it fails on a build warning, an empty chapter (a missing include) or a link that leads nowhere. `just docs` builds the book into `target/docs/html/` and `just docs serve` previews it; `just docs setup` installs mdBook, mdbook-mermaid and mdbook-llms once. Never edit anything in `target/`.

Docs-only changes use `docs:` commits, which do not bump kb-mcp's version.

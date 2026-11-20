# book/

The product book: why kb-mcp exists, what it will and will not be, where it is going, and how it compares with other projects. It is read by people deciding whether to use kb-mcp. The technical book in `docs/` is the reference; this book argues for kb-mcp and links there.

## What it may say

- **No claim the technical book does not back.** A fact about what kb-mcp does, its tools or its config is stated in `docs/src/` first. Here it is said in the reader's terms and linked.
- **Built and planned are kept apart.** Only `src/roadmap.md` describes what is not built yet, and it says so.
- No superlatives, no sales language.

## Writing a chapter

- Frontmatter `description: ...` in one line, saying who should read the chapter and when. It becomes the chapter's line in `llms.txt`.
- Add it to `src/SUMMARY.md`; an unlisted file is not in the book.
- Never hard-wrap prose in a new chapter, and keep an existing chapter's wrapping when editing it. No em dashes in new text; use a colon, a comma or a hyphen.
- A new landscape page follows `src/landscape/process.md`.

## Links into the technical book

Pages publishes the technical book at `https://ttdonovan.github.io/kb-mcp/` and this one under `/product/`. A link into the technical book is its published URL, such as `https://ttdonovan.github.io/kb-mcp/usage/configuration.html`: a relative `../` link would leave this book's `src/`. A link to a record in `docs/project/` is its GitHub URL.

## Keeping it true

When the technical book changes something a reader here would notice (a tool added, a feature built, a limit changed), update `src/introduction.md`, and `src/roadmap.md` if it was planned there, in the same change.

## Checking

Run `just book check` from the repository root after any change here. `just book` builds it into `target/book/html/` and `just book serve` previews it. The tools are the ones `just docs setup` installs. Never edit anything in `target/`.

---
name: ddg-web-search
description: >
  Search the web using DuckDuckGo Lite — no API key required.
  Triggers: "search the web", "find papers", "look up", "web search".
sources:
  - https://clawhub.ai/JakeLin/ddg-web-search
  - https://lite.duckduckgo.com/lite/
---

# DuckDuckGo Web Search

Search the web using DuckDuckGo Lite's HTML interface. No API key,
no account, no rate limits. Returns text results with titles, snippets,
and URLs.

Inspired by [JakeLin/ddg-web-search](https://clawhub.ai/JakeLin/ddg-web-search)
on ClawHub.

## How to Search

Use `web_fetch` to query DuckDuckGo Lite:

```
web_fetch(url="https://lite.duckduckgo.com/lite/?q=YOUR+QUERY+HERE")
```

### URL encoding

Replace spaces with `+` in the query string:

- `AI agent memory` → `AI+agent+memory`
- `HNSW vector search performance` → `HNSW+vector+search+performance`

### Regional filtering

Append `&kl=` for region-specific results:

| Region | Parameter |
|--------|-----------|
| US | `&kl=us-en` |
| UK | `&kl=uk-en` |
| Australia | `&kl=au-en` |
| Germany | `&kl=de-de` |

Example: `https://lite.duckduckgo.com/lite/?q=AI+memory&kl=us-en`

## Parsing Results

DuckDuckGo Lite returns plain HTML with results as numbered entries.
Each result has:

- **Title** — the linked page title
- **URL** — the destination link
- **Snippet** — a brief text excerpt

Extract the most relevant 3-5 results and their URLs.

## Follow-Up: Fetch Full Pages

For promising results, fetch the full page content:

```
web_fetch(url="https://example.com/the-result-page")
```

This gives you the complete text to synthesize into a vault entry.

## Workflow

1. Search: `web_fetch("https://lite.duckduckgo.com/lite/?q=TOPIC")`
2. Parse the numbered results, pick the best 2-3
3. Fetch each URL: `web_fetch("https://full-url-here")`
4. Synthesize findings into a vault entry with source citations

## Limitations

- No date/time filtering (DuckDuckGo Lite doesn't support it)
- Text-only results (no images or rich snippets)
- Results sourced via Bing (DuckDuckGo's upstream provider)
- Some pages may block automated fetching

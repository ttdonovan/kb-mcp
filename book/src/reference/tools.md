# Tools

## list_sections

List all collections and their sections with document counts and descriptions.

**Parameters:** None

**Returns:** JSON array of sections with `name`, `description`, `doc_count`, and `collection`.

## search

Full-text search across the knowledge base using BM25 ranking.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `query` | String | Yes | Search query (supports phrases and boolean operators) |
| `collection` | String | No | Filter by collection name |
| `scope` | String | No | Filter by section prefix |
| `max_results` | Number | No | Maximum results (default: 10) |

**Returns:** JSON with `query`, `total`, and `results` array. Each result has
`path`, `title`, `section`, `collection`, `score`, and `excerpt`.

## get_document

Retrieve a document by path or title. Content is read fresh from disk.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | String | Yes | Document path or title |

**Returns:** JSON with `path`, `title`, `tags`, `section`, `collection`, and `content`.

## kb_context

Token-efficient document briefing. Returns frontmatter metadata and first
paragraph summary without the full body.

Call this to survey relevance before using `get_document` for full content.
Saves 90%+ tokens on retrieval-heavy workflows.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | String | Yes | Document path or title |

**Returns:** JSON with `path`, `title`, `tags`, `section`, `collection`,
`frontmatter` (all fields), and `summary` (first paragraph).

## kb_write

Create a new document in a writable collection. Generates frontmatter with a
date-prefixed filename by default. Use `directory` to write into subdirectories
and `filename` to specify an exact name without date prefix.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `collection` | String | Yes | Target collection (must be writable) |
| `title` | String | Yes | Document title |
| `body` | String | Yes | Document body (markdown) |
| `tags` | List | No | Tags for frontmatter |
| `status` | String | No | Status field for frontmatter |
| `source` | String | No | Source field for frontmatter |
| `directory` | String | No | Subdirectory within collection (e.g. "concepts/memory"). Created automatically. |
| `filename` | String | No | Exact filename (e.g. "cognitive-memory-model.md"). Skips date prefix when provided. |

**Returns:** JSON with `path`, `collection`, `title`, and `tags`.

**Errors:** Returns actionable error if collection is read-only, not found, or
directory escapes the collection root.

## kb_digest

Vault summary — shows collections, sections with topics, recent additions
(last 7 days), and thin sections (fewer than 2 documents). Use this to
understand what the knowledge base covers before searching.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `collection` | String | No | Filter to a specific collection |

**Returns:** JSON with `total_documents`, `total_sections`, and `collections`
array. Each collection has `name`, `doc_count`, `sections` (with topics and
gap hints), and `recent` additions.

## kb_query

Filter documents by frontmatter fields. Multiple filters combine with AND logic.
Returns document metadata without body content.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `tag` | String | No | Filter by tag |
| `status` | String | No | Filter by frontmatter status field |
| `created_after` | String | No | YYYY-MM-DD, returns docs created on or after |
| `collection` | String | No | Filter by collection name |
| `has_sources` | Boolean | No | Only docs with a sources field |

**Returns:** JSON with `total` and `documents` array. Each document has
`path`, `title`, `tags`, `section`, and `collection`.

## kb_export

Export vault as a single markdown document. Concatenates all documents with
frontmatter headers. Use to create a portable snapshot of knowledge base content.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `collection` | String | No | Collection to export (default: all) |

**Returns:** Concatenated markdown with document separators and frontmatter metadata.

## reindex

Rebuild the search index from all collections on disk. Use after editing
documents mid-session. Note: search now auto-detects new files via directory
mtime checks, so `reindex` is mainly needed after in-place content edits.

**Parameters:** None

**Returns:** Summary message with document and section counts.

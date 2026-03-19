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

Create a new document in a writable collection. Generates proper frontmatter
with a date-prefixed filename.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `collection` | String | Yes | Target collection (must be writable) |
| `title` | String | Yes | Document title |
| `body` | String | Yes | Document body (markdown) |
| `tags` | List | No | Tags for frontmatter |
| `status` | String | No | Status field for frontmatter |
| `source` | String | No | Source field for frontmatter |

**Returns:** JSON with `path`, `collection`, `title`, and `tags`.

**Errors:** Returns actionable error if collection is read-only or not found.

## reindex

Rebuild the search index from all collections on disk. Use after adding
or editing documents mid-session.

**Parameters:** None

**Returns:** Summary message with document and section counts.

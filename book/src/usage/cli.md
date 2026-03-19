# CLI

kb-mcp works as a CLI when invoked with arguments. Every MCP tool has a
CLI equivalent.

## Commands

```sh
# List all sections
kb-mcp list-sections

# Search
kb-mcp search --query "rate limits"
kb-mcp search --query "bevy" --collection skills
kb-mcp search --query "agents" --scope runtimes
kb-mcp search --query "MCP" --max-results 5

# Get full document
kb-mcp get-document --path "concepts/mcp-server-pattern.md"

# Token-efficient briefing
kb-mcp context --path "concepts/mcp-server-pattern.md"

# Write a note (writable collection only)
kb-mcp write --collection notes --title "My Note" --body "Content" --tags "tag1,tag2"

# Rebuild index
kb-mcp reindex
```

## Global Options

| Flag | Description |
|------|-------------|
| `--config <path>` | Path to `collections.ron` config file |

## Output

All commands output JSON to stdout. Errors go to stderr.

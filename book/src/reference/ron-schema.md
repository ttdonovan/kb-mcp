# RON Schema

The full `collections.ron` schema with all fields:

```ron
(
    // Cache directory for index files
    // Default: ~/.cache/kb-mcp
    // Supports ~ expansion
    cache_dir: "~/.cache/kb-mcp",

    collections: [
        (
            // Unique name — used in search filters and kb_write target
            name: "vault",

            // Path to markdown directory
            // Relative to this config file's location
            path: "ai-vault",

            // Description shown in list_sections output
            description: "Primary knowledge vault",

            // Allow kb_write to create files here
            // Default: false
            writable: false,

            // Section definitions — map directory prefixes to descriptions
            // Documents in subdirectories matching a prefix get that section's description
            // Sections without definitions still appear, just without descriptions
            sections: [
                (prefix: "concepts", description: "Cross-cutting concepts"),
                (prefix: "guides", description: "How-to guides"),
            ],
        ),
    ],
)
```

## Rust Types

The RON file deserializes into these Rust types:

```rust
struct Config {
    cache_dir: Option<String>,
    collections: Vec<Collection>,
}

struct Collection {
    name: String,
    path: String,
    description: String,
    writable: bool,       // default: false
    sections: Vec<Section>, // default: []
}

struct Section {
    prefix: String,
    description: String,
}
```

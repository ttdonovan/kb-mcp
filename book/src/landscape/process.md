# Landscape Review Process

The AI agent memory space moves fast. This landscape section needs
periodic review to stay useful for research and feature planning.

## How to Add a New Project

1. **Clone into `sandbox/`** — gitignored, won't pollute the repo
   ```sh
   git clone --depth 1 https://github.com/org/project.git sandbox/project
   ```

2. **Run tokei** for codebase metrics
   ```sh
   tokei sandbox/project/
   ```

3. **Create a book page** at `book/src/landscape/project-name.md` with:
   - One-line description
   - Source URL and language
   - Key features list
   - Comparison table vs kb-mcp
   - Relationship (competitive, complementary, or different domain)
   - "Patterns Worth Adopting" — what we could learn from them

4. **Add to SUMMARY.md** under the Landscape section

5. **Update the overview** — add to the quick comparison table and
   codebase metrics table in `landscape/overview.md`

6. **Update `vault/tools/retrieval-landscape.md`** if the project is
   an MCP-native retrieval tool

## When to Review

- **Monthly:** Quick scan for new projects — check GitHub trending,
  Reddit r/clawdbot, ClawHub, and Hacker News for new MCP memory tools
- **Before planning a new feature:** Check if any landscape project
  already solved it — adopt patterns, don't reinvent
- **After a major release:** Update metrics and comparison tables

## What to Look For

When evaluating a new project:

| Question | Why it matters |
|----------|---------------|
| Is it MCP-native? | Direct comparison to kb-mcp's tool surface |
| Local or cloud? | Trust model and deployment alignment |
| What search does it use? | BM25, vector, hybrid, ripgrep, or none |
| Does it support write-back? | Agent-driven knowledge capture |
| What's the data model? | Files, SQLite, cloud API, knowledge graph |
| What language? | Ecosystem alignment (Rust, TypeScript, Python) |
| What's unique? | Patterns worth adopting for our roadmap |

## Regenerating Metrics

Requires [tokei](https://github.com/XAMPPRocky/tokei) (`brew install tokei`).

```sh
# Clone all landscape projects (first time only)
cd sandbox
git clone --depth 1 https://github.com/kevin-hs-sohn/hipocampus.git
git clone --depth 1 https://github.com/jimprosser/obsidian-web-mcp.git
git clone --depth 1 https://github.com/alibaizhanov/mengram.git
git clone --depth 1 https://github.com/Bumblebiber/hmem.git
git clone --depth 1 https://github.com/MadAppGang/mnemex.git
cd ..

# Run comparison
just loc-landscape
```

Update the metrics table in `landscape/overview.md` with the new numbers.

## Current Landscape (as of 2026-03-20)

| Project | GitHub | Status |
|---------|--------|--------|
| hipocampus | [kevin-hs-sohn/hipocampus](https://github.com/kevin-hs-sohn/hipocampus) | Active |
| obsidian-web-mcp | [jimprosser/obsidian-web-mcp](https://github.com/jimprosser/obsidian-web-mcp) | Active |
| mengram | [alibaizhanov/mengram](https://github.com/alibaizhanov/mengram) | Active |
| hmem | [Bumblebiber/hmem](https://github.com/Bumblebiber/hmem) | Active |
| mnemex | [MadAppGang/mnemex](https://github.com/MadAppGang/mnemex) | Active |

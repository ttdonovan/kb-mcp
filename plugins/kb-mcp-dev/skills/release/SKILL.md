---
name: release
description: Make a kb-mcp release - check the release PR that release-plz opened, its version and changelog, merge it, and confirm the tag, the GitHub Release and the binaries. Only the person starts it.
disable-model-invocation: true
---

# Make a release

How releases work is "Releases" in `CONTRIBUTING.md`, and the configuration is `release-plz.toml`. This skill is the steps and what has gone wrong before.

## Steps

1. Find the release PR: `gh pr list --search "head:release-plz-"`. If there is none, nothing releasable has merged since the last tag.
2. Check the version against the commits since the last tag (`git log --oneline $(git describe --tags --abbrev=0)..origin/main`): while the version is `0.x`, a `!` commit bumps the minor version and anything else the patch version (`CONTRIBUTING.md`, Versioning).
3. Check `CHANGELOG.md` in the PR against the same commits: each `feat:` and `fix:` of all three crates is there, worded for a user. To change it, commit on the PR's branch; the edit lasts only until the next merge to `main`.
4. Show the person the version and the changelog. Merging is theirs. The release PR is opened by GitHub Actions, so CI does not run on it on its own and the `check` requirement needs their bypass; pushing a commit to its branch, as in step 3, runs CI.
5. After the merge, confirm the release: `gh release view vX.Y.Z --json assets --jq '.assets[].name'` lists a tarball for each of the four targets and `SHA256SUMS`. If a binary build failed, the tag and the release still stand: fix the workflow on a branch, then run "Release binaries" by hand with the tag (`gh workflow run release-binaries.yml -f tag=vX.Y.Z`).

## Gotchas

- **Merge nothing else while a release PR is open.** Every push to `main` regenerates it, which drops an edit made in step 3. Merge the release PR first.
- **The changelog misses commits** when a crate's previous release cannot be found: release-plz reads it from the `vX.Y.Z` tag, through `git_tag_name` in `release-plz.toml`. If kb-core or kb-cli commits are missing, check the release-plz job's log for "Latest release of package".
- **Dependabot PRs** are merges to `main` like any other. Merge them before starting a release, not during one.

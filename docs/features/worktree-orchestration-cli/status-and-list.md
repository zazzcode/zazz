# CLI Capability: `zaz status` and `zaz list`

## Purpose

These commands make the managed worktree graph legible to humans and agents.

## Main-Doc Authority

Authoritative capability section:

- `C3` Status, listing, inspection, and graph visibility

Authoritative feature doc:

- `docs/features/worktree-orchestration-cli.md`

## Capability Summary

`zaz status` and `zaz list` should surface:

- managed worktrees and branches
- parent or upstream relationships
- readiness and freshness state
- stale relative to parent vs stale relative to integration base
- conflict or recovery-required state
- machine-readable summaries for agents

## UX Direction

These commands should:

- be the fastest way to answer "what is going on in this repo?"
- work well in both human-readable and JSON output modes
- expose enough structure that a later UI can consume the same truth model

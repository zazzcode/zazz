# CLI Capability: `zaz add`

## Purpose

`zaz add` is the primary managed worktree creation command for day-to-day graph growth.

## Main-Doc Authority

Authoritative capability sections:

- `C2` Managed worktree lifecycle and materialization
- parts of `C4` Sync, refresh, propagation, and merge-order resilience

Authoritative feature doc:

- `docs/features/worktree-orchestration-cli.md`

## Command Direction

Current direction:

- `zaz add <name> [--from <parent>]`

M1 direction:

- support creation from the local integration source and managed parent flows within the MVP scope
- preserve zazzles naming and stack-suffix rules

## Capability Summary

`zaz add` should:

- create one real branch and one real worktree
- register the node in `.zazz/`
- prepare the new worktree for immediate use
- materialize configured untracked files
- preserve stack naming behavior without turning suffixes into dependency truth
- support safe cleanup through the broader worktree lifecycle

## Related Operations

This capability family also includes:

- worktree removal
- shared excludes validation
- repo-specific setup after worktree creation

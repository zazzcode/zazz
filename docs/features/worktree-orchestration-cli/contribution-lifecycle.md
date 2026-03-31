# CLI Capability: Contribution Lifecycle

## Purpose

This capability family makes a managed worktree a full contribution unit from commit through PR update.

## Main-Doc Authority

Authoritative capability section:

- `C7` Contribution lifecycle and PR operations

Authoritative feature doc:

- `docs/features/worktree-orchestration-cli.md`

## Command Families

Planned command families include:

- `zaz commit ...`
- `zaz push ...`
- `zaz pr ...`

## Capability Summary

This capability family should:

- commit with profile-aware identity defaults
- push with safe remote behavior
- create and update draft or ready-for-review PRs
- preserve parent-relative PR targeting
- mirror relevant PR state into `.zazz/`

## User Value

This is the point where zazzles stops being only a local worktree manager and becomes a full CLI surface for graph-aware contribution workflows.

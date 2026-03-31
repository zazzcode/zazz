# CLI Capability: Existing Compatible Layout Adoption

## Purpose

This capability allows zazzles to adopt an already-existing compatible `.bare/` plus sibling-worktree repo container into managed `.zazz/` state without recloning or rebuilding the local layout.

## Main-Doc Authority

Authoritative capability section:

- `C1` Repo bootstrap and existing-layout adoption

Authoritative feature doc:

- `docs/features/worktree-orchestration-cli.md`

## Capability Summary

The adoption flow should:

- validate that the existing repo container is structurally compatible
- detect or confirm repo identity and the integration worktree
- initialize `.zazz/` state without recreating the Git layout
- import visible worktrees into an initial trustworthy registry
- expose ambiguities explicitly instead of inventing graph truth silently

## Safety Rules

Adoption should:

- be additive and low-risk
- reject incompatible layouts rather than attempting best-effort normalization
- keep imported graph truth conservative when parentage cannot be inferred safely

## User Value

This capability matters because it lowers adoption friction for teams already using the `.bare/` pattern manually or through other tooling.

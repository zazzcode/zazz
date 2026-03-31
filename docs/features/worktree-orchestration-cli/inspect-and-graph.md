# CLI Capability: `zaz inspect` and `zaz graph`

## Purpose

These commands provide focused node detail and graph-oriented visibility beyond basic list output.

## Main-Doc Authority

Authoritative capability section:

- `C3` Status, listing, inspection, and graph visibility

Authoritative feature doc:

- `docs/features/worktree-orchestration-cli.md`

## Capability Summary

`zaz inspect` and `zaz graph` should help a user understand:

- one node's parent, children, ancestry, and readiness
- intended dependency vs current operational ancestry
- graph shape across stacks and DAG-like flows
- why a node is stale, blocked, or ready

## User Value

These commands are important because a worktree list alone does not explain dependency shape, convergence readiness, or propagation impact.

# CLI Capability: `zaz sync`

## Purpose

`zaz sync` is the command family that keeps the managed graph convergent as local parents and the remote integration branch move.

## Main-Doc Authority

Authoritative capability section:

- `C4` Sync, refresh, propagation, and merge-order resilience

Authoritative feature doc:

- `docs/features/worktree-orchestration-cli.md`

## Capability Summary

`zaz sync` should:

- refresh worktrees against the configured integration branch
- propagate upstream changes through downstream nodes in dependency order
- handle out-of-order merge completion
- classify stale state clearly
- produce durable, machine-readable results for follow-up automation

## Important Distinction

This capability is broader than "run rebase on all branches."

It must understand:

- local parent dependencies
- integration-branch drift
- downstream blast radius
- graph ordering and convergence behavior

# CLI Capability: `zaz conflicts` and `zaz resolve`

## Purpose

These commands turn failed refresh or merge operations into durable, recoverable workflows.

## Main-Doc Authority

Authoritative capability section:

- `C5` Conflict capture, recovery, and resolution handoff

Authoritative feature doc:

- `docs/features/worktree-orchestration-cli.md`

## Capability Summary

This command family should:

- persist conflict artifacts in `.zazz/`
- classify the failed operation and affected files
- provide a machine-readable handoff contract
- support AI-assisted or human-assisted recovery
- reconcile resolution results back into graph truth

## User Value

Without this capability, refresh failures collapse back into ad hoc terminal state and the orchestration product loses one of its main advantages.

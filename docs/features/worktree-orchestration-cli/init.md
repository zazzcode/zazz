# CLI Capability: `zaz init`

## Purpose

`zaz init` is the greenfield bootstrap command for creating a new managed repo container in the zazzles `.bare/` plus sibling-worktree model.

## Main-Doc Authority

Authoritative capability sections:

- `C1` Repo bootstrap and existing-layout adoption
- `C2` Managed worktree lifecycle and materialization

Authoritative feature doc:

- `docs/features/worktree-orchestration-cli.md`

## Command Direction

Current direction:

- `zaz init <repo-name> [--integration <branch>]`

Current M1 constraints:

- repo-name-first invocation
- fresh bootstrap only
- integration branch resolution from explicit flag, user-global default, then fallback
- readiness checks before local state mutation

## Capability Summary

`zaz init` should:

- validate local prerequisites and provider readiness
- create the managed repo root and `.bare/` structure
- create the integration worktree
- initialize `.zazz/` state safely
- seed repo-local configuration required by later commands
- prepare shared excludes and initial materialization state

## State Impact

Repo-local state:

- initializes `.zazz/`
- establishes the initial worktree registry and graph truth

User-global state:

- may read user-global defaults
- should not require interactive mutation of user-global config in the bootstrap path

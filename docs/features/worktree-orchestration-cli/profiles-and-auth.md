# CLI Capability: Profiles and Auth

## Purpose

This capability family supports multi-account, multi-organization, and sandbox-safe authenticated workflows.

## Main-Doc Authority

Authoritative capability section:

- `C6` Profiles, auth, identity, and directory-scoped defaults

Authoritative feature doc:

- `docs/features/worktree-orchestration-cli.md`

## Command Families

Planned command families include:

- `zaz account ...`
- `zaz auth ...`
- `zaz profile ...`
- `zaz config ...`

## Capability Summary

This capability family should provide:

- named host/account profiles
- Git author identity defaults per profile
- secure PAT handling without repo-local secret persistence
- directory-scoped default profiles
- deterministic profile precedence
- non-interactive auth handoff for sandboxed agents

## Differentiation Value

This is one of the clearest opportunities for zazzles to differ from adjacent tools for users who work across many clients, orgs, or gigs on one machine.

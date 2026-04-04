# AGENTS.md

This repository uses the Zazz framework for durable product docs, repo standards, and worktree-oriented execution.

## Agent Directive Callout Standard

Use this callout format across project docs when an instruction is specifically for agents and must be treated as normative:

`⚠️ Agent Directive ⚠️`

Definition:

- `⚠️ Agent Directive ⚠️` marks binding agent-facing instructions.
- Requirement levels in Agent Directive lines use uppercase keywords: `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, `MAY`.
- If a conflict exists, Agent Directive lines take precedence over nearby non-directive guidance in the same document.

Examples:

- `⚠️ Agent Directive ⚠️ MUST NOT: If instructions are unclear or conflicting, do not guess; ask for clarification.`
- `⚠️ Agent Directive ⚠️ MUST NOT: When addressing PR feedback, do not modify files outside the PR's changed files unless explicitly instructed.`

## Docs Root

`Framework docs root: docs`

## Standards Loading Rules

Authoritative standards index:

- `docs/standards/index.yaml`

Required behavior:

1. Read `docs/standards/index.yaml` before making material code, architecture, review, or documentation changes.
2. Load only the standards that apply to the task.
3. Prefer the repo standard over ad hoc local patterns when they conflict.

## Feature Context Rules

Feature index:

- `docs/features/index.yaml`

Current repo state:

- Feature docs are not yet fully authored.
- Use the proposal in `docs/proposals/` as the primary product-context document until formal feature docs are approved.

## Deliverables Policy

Deliverable docs live under:

- `docs/deliverables/`

Default policy:

- deliverable SPECs and PLANs are worktree-local execution artifacts
- durable docs under `docs/proposals/`, `docs/features/`, and `docs/standards/` are tracked
- deliverable files should only be committed when intentionally shared as part of reviewable project knowledge

## Tracking System Policy

Tracking system:

- No external tracker is configured yet.
- GitHub branches, PRs, and durable docs are the current source of execution context.
- Do not invent Jira, Zazz Board, or other ticket identifiers for this repo.

## Shared-File Coordination Policy

Shared-file coordination:

- No external locking tool is declared for this repo.
- Use harness-native isolation when available.
- Serialize overlapping-file work when safe isolation is not guaranteed.

## Worktree Policy

Project-specific worktree rules for this repo:

- `main` is the integration branch.
- ⚠️ Agent Directive ⚠️ MUST NOT: Make code, docs, or config edits directly in the integration worktree (typically `main`).
- ⚠️ Agent Directive ⚠️ MUST: Perform implementation and documentation edits in a non-integration feature worktree, then merge via PR.
- Use one active feature, deliverable, or proposal branch per worktree.
- Prefer flat branch names with hyphens instead of `/` when working inside this repo.
- For a new development branch, branch from the synced local `main`.
- For PR review or continuing existing work, create the worktree from the existing remote branch instead of branching from local `main`.
- Merge through PR review rather than local integration merges.

## Repo Orientation

Important repo entry points:

- `README.md`
- `docs/proposals/worktree-native-pr-orchestrator.md`
- `docs/standards/repo-foundations.md`

Imported framework skills:

- `.agents/skills/` was copied from the sibling `zazz-skills` repository so proposal, feature, planning, worker, QA, and PR workflows are available locally.

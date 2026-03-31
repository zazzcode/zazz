# Feature: zazzles App and UI

## Feature Summary

The zazzles App and UI is the installable local application that gives humans a rich visual surface for graph inspection, review, conflict handling, and guided worktree operations on top of the shared zazzles core engine.

This feature exists so users can understand and operate complex stacked and DAG-shaped workflows without relying only on terminal output or GitHub's flat PR lists. It is the primary human-facing visual experience for the product, but it remains a companion to the CLI rather than a replacement for it.

## Current Milestone / Next Milestone / Services Affected

- Current milestone: M0 Proposed
- Next milestone: M1 App Shell and Graph Readiness
- Services affected:
  - local app host/runtime
  - shared orchestration core
  - local graph and worktree state in `.zazz/`
  - local diff/review UI surfaces
  - GitHub-aware review context
  - future local daemon or streaming state source

## Introduction

This feature defines the long-lived application and UI capability of the product.

The naming direction for this feature is intentional:

- prefer "app" over "desktop" in durable product and repo language
- the value is the self-contained local application experience, not the specific hardware form factor
- the repo implementation directory should use `apps/client/` as the neutral local application host location

The app should work naturally on laptops and desktops, but the product concept is broader than a device category. The important distinction is that this is the installable visual application surface built on the same core engine as `zaz`.

## Why This Feature Matters

- Humans need a graph-native review and orchestration surface once workflows become more complex than a simple sequential stack.
- The app can make dependency shape, blocked nodes, stale descendants, and convergence readiness legible at a glance.
- Parent-relative diffs and graph-aware review are core product differentiators that are awkward to express through terminal output alone.
- A strong local app can reduce the cognitive load of managing multiple worktrees and agent outputs on one machine.
- The app provides the natural home for richer conflict review and guided acceptance flows that would be clumsy in the CLI.

## Current State

Current state today:

- no installable app implementation exists yet
- the product direction for a visual application has been described in the proposal
- the Rust workspace reserves an `apps/client/` location for the future application host
- the CLI feature is being defined first as the foundational execution surface

What is not live yet:

- no app shell
- no graph UI
- no embedded diff/review surface
- no app-level conflict UX
- no client-side local-only-file diff-and-accept workflow

## Relationship To The CLI

This feature depends on the CLI and shared orchestration core.

Principles:

- the CLI remains the authoritative automation surface
- the app should consume shared logic and durable state rather than reimplementing orchestration rules independently
- the app may initiate actions that are also available through the CLI, but it should do so through shared core behavior and stable command/state contracts
- the app should make complex state easier to understand, not create a second incompatible operational model

## Implementation Stack Direction

The current implementation direction for this feature is:

- Rust for the local app host and shared native integration layer
- Tauri as the preferred installable application shell
- a web-based UI layer for graph, review, and diff experiences
- shared Rust crates for orchestration, graph logic, and repo state access
- machine-readable CLI/state contracts that the app can consume or mirror

Why this direction is preferred:

- it preserves a shared engine with the CLI
- it matches the repo standards that favor a shared core consumed by both CLI and UI layers
- it is a pragmatic path to polished graph and diff experiences without forcing a pure-Rust UI too early

## Core Concepts

### Local app

The installable zazzles application that runs on the user's machine and provides the primary visual experience for the product.

### Graph view

A UI surface that shows worktree nodes, edges, PR state, freshness, blocked status, and convergence relationships in a way that humans can scan quickly.

### Review surface

A UI flow for understanding one node relative to its parent, including diffs, branch metadata, readiness, and related upstream/downstream context.

### Conflict surface

A guided UI for inspecting failed refresh or merge operations, understanding what changed, and moving into either resolution or escalation paths.

### Local-only file maintenance flow

A future app-oriented flow that helps the user compare local-only untracked files in the current worktree against the integration worktree and decide what should become the canonical source for future worktree creation.

For M1 of the CLI feature, this remains manual. The app feature is the natural long-term home for that workflow.

## User Flows and System Flows

### Primary app flow

1. Launch the zazzles app for a repo container.
2. Load repo-local `.zazz/` state and current worktree metadata.
3. Render the graph of managed worktrees and their statuses.
4. Let the user inspect one node in detail.
5. Let the user move into diff, review, or conflict-recovery surfaces.
6. Hand off orchestration actions to shared core logic rather than duplicating workflow rules in the UI.

### Graph review flow

1. User selects a worktree node.
2. The app shows parent, children, readiness, PR state, and freshness context.
3. The app presents a parent-relative diff or summary.
4. The user decides whether the node is ready, blocked, stale, or needs intervention.

### Local-only file maintenance flow

This is explicitly a later milestone flow, not an initial app milestone:

1. User opens a local-only file maintenance view for the current worktree.
2. The app diffs local-only untracked files in the current worktree against the integration worktree.
3. The app shows which files differ and how.
4. The user chooses which updates should be accepted back into the integration-worktree source of truth.
5. The system applies the accepted updates with clear reviewability and auditability.

## Feature-Level Success Criteria

This feature is successful when:

- the app makes the active worktree graph easier to understand than CLI output alone
- humans can inspect a node's graph context and parent-relative change set without piecing it together manually
- the app reuses the shared core logic rather than drifting into a separate workflow model
- the app becomes the natural home for graph-aware review and conflict experiences
- future local-only-file maintenance can be handled through a visual diff-and-accept workflow instead of only manual copying

## Milestone Overview

| Milestone | Status | Outcome |
| --- | --- | --- |
| M1 | Proposed | App shell, repo loading, graph-ready state ingestion, and foundational UI structure |
| M2 | Proposed | Graph review and node inspection experience |
| M3 | Proposed | Diff, conflict, and local-only-file maintenance experiences |

## Milestone Details

### M1: App Shell and Graph Readiness

Outcome criteria:

- the repo has a clear application host location under `apps/client/`
- the app can launch locally and target the repo container model
- the app can load enough repo-local state to understand the current managed-worktree graph
- the app establishes foundational navigation, layout, and state-loading patterns for later graph and review work

Likely deliverables:

- app host bootstrap
- shared-core state loading integration
- initial repo selection or repo binding behavior
- foundational UI shell

### M2: Graph Review and Node Inspection

Outcome criteria:

- the app can render a readable graph of worktree relationships
- a user can inspect node details without dropping to the terminal
- the app can show parent/child context, freshness, and readiness state clearly

Likely deliverables:

- graph canvas
- node detail panel
- status and readiness visualization
- graph filtering and selection basics

### M3: Diff, Conflict, and Local-Only File Maintenance

Outcome criteria:

- the app can present parent-relative diffs in a user-friendly review flow
- the app can surface conflict artifacts and guide users into resolution paths
- the app can support a visual diff-and-accept workflow for local-only untracked files against the integration worktree

Likely deliverables:

- embedded diff/review surface
- conflict review UX
- local-only file comparison view
- selective accept/write-back flow for updates to the integration-worktree source of truth

## Planned Future Evolution

- richer DAG visualization should layer on top of the CLI and shared core once the core orchestration model is proven
- the app should eventually support live state updates from a daemon or streaming event source
- conflict handling can evolve from read-only visibility into guided approval and resolution flows
- local-only-file maintenance should become a first-class visual workflow in the app once the basic CLI materialization model is stable
- future AI assistance may help summarize diffs, explain graph changes, or propose conflict resolutions within the app

## Risks, Constraints, and Non-Goals

Risks:

- UI work can drift ahead of the underlying orchestration truth if the shared core contracts are weak
- graph-heavy interfaces can become visually noisy if the information hierarchy is not intentional
- embedding diff and conflict experiences can grow into a large surface area quickly

Constraints:

- the CLI remains the source of automation truth
- the app should consume stable repo-local state and shared core behavior
- the initial app should not require inventing a second orchestration engine

Non-goals:

- replacing the CLI as the primary automation surface
- forcing users into a UI-only workflow
- shipping every advanced review or conflict workflow in the first application milestone

## Open Questions

- how much app behavior should directly invoke shared Rust core APIs versus consuming CLI-level command contracts?
- what is the right first graph visualization scope before diff and conflict surfaces are added?
- what should the acceptance and write-back UX look like for future local-only-file maintenance against the integration worktree?
- should the initial app be repo-bound to one container at a time, or support switching between multiple local repos early?

## Deliverable Handoff Considerations

The first deliverables for this feature should likely start with:

- application host scaffolding in `apps/client/`
- shared-core integration for loading repo-local state
- a basic shell that can display repo and worktree graph readiness

This feature should remain dependent on the CLI feature's core state and orchestration rules rather than replacing them.

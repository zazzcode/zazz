# Feature: Worktree Orchestration CLI

## Feature Summary

The Worktree Orchestration CLI is the primary execution surface for managing local worktree graphs, parent-relative PR workflows, automated propagation, and agent-friendly orchestration on a single machine.

This feature exists so humans and agents can operate the product without depending on a desktop UI. It is the foundational user-facing capability that proves the product model before richer visualization and review experiences are layered on top.

At its core, the CLI exists to manage the chaos of out-of-order PR review and merge by keeping the local worktree graph coherent and by applying automation plus AI assistance to the propagation and conflict work that follows.

## Current Milestone / Next Milestone / Services Affected

- Current milestone: M0 Proposed
- Next milestone: M1 CLI MVP
- Services affected:
  - local Git repository and worktree container
  - local `.zazz/` state
  - GitHub PR integration
  - remote integration branch state
  - local AI-assisted conflict workflows

## Introduction

This feature defines the long-lived CLI capability of the product.

The CLI is not just an implementation convenience. It is the durable interface through which:

- agents perform worktree and graph operations
- humans inspect and operate the graph without needing the desktop app
- the team proves the product's automation model in real usage before UI polish is added

Under the Zazz framework, this is a valid feature because it describes a durable application capability over time, has clear milestone evolution, and is expected to remain part of the product even after the desktop app exists.

## Implementation Stack Direction

The current implementation direction for this feature is:

- Rust for the shared orchestration engine and the CLI application
- system `git` invocation for branch, worktree, fetch, rebase, push, and cleanup operations
- GitHub integration through a dedicated adapter layer, with optional `gh` interoperability where that reduces setup friction
- repo-local state in `.zazz/`
- user-global product state in `~/.zazz/worktrees/`
- machine-readable JSON output for agent-facing operations

Important clarification:

- this feature is not centered on GitHub CI as the primary execution model
- the core behavior runs locally on the user's machine
- GitHub is primarily the remote host and PR system for this feature
- GitHub Actions or other CI systems may validate branches later, but they are not the orchestration engine

Why this direction is preferred:

- Rust gives the project a strong foundation for a durable local engine and a reliable CLI
- calling system `git` in v1 is lower risk than re-implementing Git internals
- a dedicated GitHub adapter keeps PR awareness explicit without making the CLI dependent on a hosted control plane
- this keeps the CLI aligned with the proposal's broader Rust-first architecture while remaining practical to implement

## Feature Overview Diagrams

### CLI System Components

```mermaid
flowchart LR
    U["Human or Agent"] --> CLI["Worktree CLI"]
    CLI --> GE["Git Interaction Engine"]
    CLI --> DAG["Local .zazz State"]
    CLI --> GH["GitHub Adapter"]
    CLI --> AI["AI Conflict Services"]
    GE --> GIT["System Git"]
    DAG --> FS["Local Filesystem"]
    GH --> REMOTE["GitHub"]
```

### M1 CLI Foundation Flow

```mermaid
flowchart TD
    A["Run CLI doctor/init"] --> B{"Repo container ready?"}
    B -->|No| C["Guide bootstrap or adoption"]
    B -->|Yes| D["Initialize .zazz state"]
    C --> D
    D --> E["Create worktree node from parent branch"]
    E --> F["Materialize local-user files and settings"]
    F --> G["Register node and edge in DAG state"]
    G --> H["List and inspect graph status"]
    H --> I["Return human-readable and JSON output"]
```

### M1 Worktree Creation Sequence

```mermaid
sequenceDiagram
    participant U as Human or Agent
    participant CLI as Worktree CLI
    participant DAG as .zazz State
    participant GIT as System Git
    participant FS as Filesystem
    participant LP as Local User Profile

    U->>CLI: create node from parent branch
    CLI->>CLI: validate prerequisites and repo topology
    CLI->>DAG: initialize or load local graph state
    CLI->>GIT: create branch and worktree
    GIT->>FS: materialize sibling worktree directory
    CLI->>LP: copy or link local-only files and settings
    CLI->>DAG: register node, parent, path, and status
    CLI->>U: return node status and graph summary
```

### Remote Base Refresh Flow

```mermaid
sequenceDiagram
    participant CLI as Worktree CLI
    participant GIT as System Git
    participant DAG as .zazz State
    participant WT as Managed Worktrees

    CLI->>GIT: fetch remote integration branch
    GIT->>CLI: report new integration-base commit
    CLI->>DAG: mark affected nodes stale
    CLI->>WT: refresh all affected worktrees against remote base
    CLI->>DAG: update freshness and operational ancestry state
    CLI->>CLI: report which nodes are current, conflicted, or blocked
```

## Why This Feature Matters

- Agents will always need a scriptable surface.
- The hardest product questions are orchestration questions, not UI questions.
- A CLI-first path lets the team validate worktree creation, PR lifecycle, propagation, and conflict behavior early.
- The CLI can be dogfooded while the product itself is being built.
- The desktop application can later become a companion feature built on top of a trusted engine and CLI workflow.
- The CLI is the first place where the product proves it can absorb unpredictable PR timing and keep downstream work moving anyway.

## Current State

Current state today:

- no implementation exists yet
- the product intent, architecture direction, and MVP sequencing have been drafted in the proposal
- the repo has the initial Zazz docs scaffold, proposal, standards index, and imported skills

What is not live yet:

- no CLI commands
- no local `.zazz/` graph state
- no worktree bootstrap logic
- no GitHub PR orchestration
- no propagation engine

## Feature-Level Success Criteria

This feature is successful when:

- a human or agent can operate the core product entirely through the CLI
- the CLI can create and manage the local worktree DAG on one machine
- the CLI can open and track parent-relative draft or ready-for-review PRs
- merged or updated upstream branches propagate forward through downstream branches with minimal manual work
- AI-assisted conflict handling is available when propagation is not clean
- the CLI remains the authoritative automation surface even after the desktop application ships
- the CLI reliably handles out-of-order review and merge events without forcing humans to manually babysit the graph

## Core Concepts

### Worktree node

A managed unit consisting of:

- a branch
- a local worktree path
- local DAG metadata
- optional PR state
- optional agent ownership state

The node identity should not depend on a numeric suffix in the branch or worktree name. Human-meaningful names are preferred, and uniqueness can be handled separately from dependency truth.

### Parent source

When creating a new managed worktree node, the parent may be:

- another local managed worktree node
- the configured remote integration branch such as `origin/main` or `origin/dev`

The CLI should support both patterns and track them consistently inside the same local DAG/state model.

### Local user profile materialization

The process of copying, linking, or otherwise materializing local-only user files into a newly created worktree so that the worktree is actually usable immediately.

Examples may include:

- `.env` files
- local editor or agent settings
- cached local-only support files
- other explicitly configured per-user or per-machine files required for normal development

This matters because standard Git branching keeps a user in one working directory, while managed worktrees create a new directory that would otherwise be missing those local-only files.

Materialization source rules:

- if a new worktree is created from another local managed worktree, local-only files should be materialized from that source worktree
- if a new worktree is created from the configured remote integration branch, local-only files should be materialized from the local integration worktree for `main` or `dev`
- the local integration worktree should therefore be treated as a stable, always-runnable source of truth for origin-rooted worktree creation

### Graph edge

A dependency relationship indicating that one worktree node depends on another node's branch state.

### Dynamic DAG

The dependency graph is DAG-capable, but it is not assumed to be static. As PRs are reviewed, merged, refreshed, or reparented, the active operational graph may change.

### Intended dependency vs operational ancestry

The feature should distinguish between:

- intended dependency intent, meaning how units of work conceptually relate
- current operational ancestry, meaning which branch state a node is currently based on after merges, rebases, refreshes, or reparenting

This distinction matters because merge order may force the operational graph to shift even when the conceptual work breakdown remains the same.

### Propagation

The process of refreshing downstream nodes after upstream branches are updated or merged.

Within this feature, propagation is one of the central responsibilities of the CLI rather than a secondary convenience.

### Remote integration drift

The rule that local worktrees may become stale not only because of local ancestor changes, but also because the configured remote integration branch has advanced due to teammates merging other work.

The CLI must therefore:

- detect remote base movement
- show which nodes are stale relative to that base
- support refresh or rebase operations against the configured remote integration branch

### Conflict artifact

A repo-local record of a failed refresh or rebase that captures enough detail for a human or agent to continue the recovery flow deliberately.

A conflict artifact should include at least:

- the worktree path and branch
- the command or operation that was attempted
- the integration base or parent that was being applied
- the conflicting files
- the conflict status and next recommended action
- a stable artifact path that another agent can be pointed at

Conflict artifacts should live in repo-local orchestration state rather than only in transient terminal output.

### Repo-local versus user-global state

The feature should separate:

- repo-local orchestration state in `.zazz/` for DAG metadata, worktree registry, conflict artifacts, and operation state tied to one repository
- user-global state in `~/.zazz/` for machine-level preferences, provider credentials, caches, logs, reusable agent settings, and product-family configuration

The DAG and per-repo conflict state should remain canonical inside `.zazz/`.

The same product name is used in both places, but the scope is determined by location:

- `.zazz/` inside the repo container is repo-local orchestration state
- `~/.zazz/` in the user home directory is user-global Zazz family state

Recommended user-global layout:

```text
~/.zazz/
├── worktrees/
│   ├── config.toml
│   ├── providers.toml
│   ├── logs/
│   └── cache/
├── board/
│   ├── config.toml
│   └── cache/
└── shared/
    ├── auth/
    └── profiles/
```

Recommended repo-local layout:

```text
repo-container/
├── .bare/
├── .zazz/
│   ├── config.toml
│   ├── graph.json
│   ├── worktrees.json
│   ├── conflicts/
│   └── locks/
├── main/
└── feature-worktree/
```

### Graph-aware merge order

The rule that creation order does not guarantee review or merge order. The CLI must follow actual dependency relationships and current merged state, not assume that lower-numbered or earlier-created nodes always complete first.

In practice, this works like a normal human team:

- whichever PR gets reviewed and merged first may simply be the one that got attention first
- the CLI must therefore treat out-of-order merge completion as routine and automatically drive the required downstream refresh behavior

This is the main operational value of the feature, not just an edge case to support.

### Draft PR checkpoint

A branch-level review checkpoint that allows human visibility before final merge readiness.

### Human ownership boundary

AI and agents may implement, synchronize, and propose resolutions, but a human remains accountable for approval and merge.

## User Flows and System Flows

### Primary CLI flow

1. Initialize or adopt a repo container.
2. Create a worktree node from a chosen parent.
3. Register it in `.zazz/`.
4. Open a draft PR against the parent branch.
5. Continue downstream work in additional nodes.
6. Detect upstream updates or merges.
7. Propagate those changes downstream.
8. Resolve conflicts with AI assistance or semantic migration when needed.
9. Detect when the remote integration branch has advanced and refresh affected nodes as needed.

Important flow nuance:

- node creation order does not guarantee PR completion order
- smaller downstream branches may be reviewed and merged before larger earlier-created branches
- the CLI must therefore compute propagation from the actual dependency graph and current branch state, not from naming sequence alone
- when merge order changes the effective branch ancestry, the CLI must be able to refresh or reshape the active operational graph without losing track of the original work intent
- the CLI must also account for remote branch movement caused by other developers working in the same repository

### Status and worktree list flow

The CLI should provide a `list` or `status` style command that shows, for each managed worktree node:

- branch and worktree path
- parent dependency
- PR state when present
- whether it is current relative to its parent
- whether it is stale relative to the configured remote integration branch
- whether it needs refresh, conflict resolution, or migration

This status view should include both:

- worktrees that branch from other local managed nodes
- worktrees that branch directly from the configured remote integration branch

### Recovery flow

1. A refresh or rebase operation fails with conflicts.
2. CLI marks the node as conflicted or migration-required.
3. CLI saves a repo-local conflict artifact for that failed operation.
4. CLI returns machine-readable conflict status, including the artifact path and affected worktree.
5. A human or orchestrator points an agent at that artifact and worktree.
6. The agent resolves directly in the worktree or proposes a resolution.
7. CLI reconciles the result and refreshes DAG state.

## CLI Vocabulary and Surface Direction

The public command should be `zazz`. The CLI should feel familiar to users who already know `git` and `gh`, while still exposing Zazz-specific graph and orchestration behavior clearly.

Direction:

- use `zazz` as the top-level command
- make `zazz --help` the primary discoverability entry point for humans
- keep the command families Git-like and verb-first where possible
- prefer human-readable defaults with `--json` for agent consumption
- make help and error output part of the contract, not an afterthought

Milestone 1 should explicitly define these initial command families:

- `zazz init`
- `zazz status`
- `zazz worktree ...`
- `zazz graph ...`
- `zazz sync ...`
- `zazz conflicts ...`
- `zazz resolve ...`

Illustrative command shape:

```sh
zazz --help
zazz init
zazz status
zazz worktree add rbac-mvp-auth-foundation --from main
zazz worktree add rbac-mvp-oauth --from rbac-mvp-auth-foundation
zazz worktree list
zazz graph inspect rbac-mvp-oauth
zazz sync refresh --all
zazz sync refresh rbac-mvp-oauth
zazz conflicts show rbac-mvp-oauth
zazz resolve rbac-mvp-oauth
```

The exact final command families may still evolve, but Milestone 1 should explicitly define the public CLI vocabulary rather than leaving it implicit in implementation.

## Milestone Overview

| Milestone | Status | Outcome |
| --- | --- | --- |
| M1 | Proposed | CLI foundations for managed worktree creation, local DAG truth, origin refresh, conflict capture, and agent-assisted origin-refresh resolution |
| M2 | Proposed | PR lifecycle automation plus parent-to-child propagation after upstream merge or update |

## Milestone Details

### M1: CLI MVP

Outcome criteria:

- the local DAG/state model is clearly defined and initialized in a durable repo-local location
- the public `zazz` command surface is defined clearly enough that `zazz --help` provides a stable entry point for both humans and agents
- users can initialize the required repo/worktree topology, with readiness checks performed as part of `zazz init`
- a local integration worktree for `main` or `dev` is kept available as the stable origin-rooted creation source
- users can create a new managed worktree that is actually usable immediately
- required local-only files and settings are materialized automatically
- users can create and inspect local DAG nodes through the CLI
- users can see which worktrees are current or stale relative to their parent and configured integration base
- users can create a worktree from either a local managed parent node or the configured remote integration branch
- remote integration-branch movement can refresh affected worktrees automatically
- refresh conflicts are captured as durable repo-local artifacts that can be handed to another agent or a human
- agent-assisted conflict handling exists for origin-refresh failures
- the CLI can represent enough DAG semantics to handle local-parent flows, remote-rooted flows, parallel descendants, and convergence readiness
- the CLI exposes machine-readable output suitable for agent use

M1 boundary:

- this milestone is the first fully useful CLI milestone
- it should be valuable on its own for real day-to-day worktree usage
- it should establish the full CLI MVP before the desktop application is pursued as a separate feature

Proposed M1 command surface:

- `zazz --help`: primary discoverability entry point for humans and the top-level summary of the CLI contract.
- `zazz init`: perform prerequisite checks, validate or adopt the repo/container topology, initialize local `.zazz/` state, and ensure the local integration worktree convention is established.
- `zazz status`: show repo-level Zazz health, managed-worktree summary, and whether the repo appears ready for normal CLI operations.
- `zazz worktree add <name> --from <local-node-or-origin-base>`: create a new managed worktree from either a local managed node or the configured origin-rooted integration branch, materialize local-only files, and register the node in `.zazz/`.
- `zazz worktree list`: list all managed worktrees for the repo with high-level freshness and readiness information.
- `zazz worktree inspect <name>`: show detailed state for one managed worktree, including path, origin, intended parent, operational base, and current status.
- `zazz worktree remove <name>`: safely remove a managed worktree and update local orchestration state when removal is allowed.
- `zazz graph inspect <name>`: inspect the graph relationships around a worktree, including parent, children, and ancestry-related metadata.
- `zazz sync refresh --all`: refresh all eligible managed worktrees in the repo against the configured remote integration branch.
- `zazz sync refresh <name>`: refresh one managed worktree against the configured remote integration branch.
- `zazz conflicts show <name>`: display the saved conflict artifact and recovery context for a worktree whose origin refresh failed.
- `zazz resolve <name>`: run or resume the agent-assisted resolution flow for a worktree with a saved origin-refresh conflict.

Command-shape notes:

- `zazz init` should perform prerequisite and topology checks before making changes, rather than requiring a separate preflight-only command in M1
- `zazz worktree add` is the main M1 creation command and should intentionally mirror `git worktree add` muscle memory as closely as the product's extra orchestration requirements allow
- unlike raw Git, Zazz may derive the worktree path from the managed repo layout, so the primary positional argument should be the managed worktree name and the source branch or node should be expressed explicitly with `--from`
- the M1 command surface is intentionally focused on local graph truth, worktree usability, origin refresh, and conflict recovery rather than PR lifecycle automation

The exact flags may still evolve during spec work, but this command set should be treated as the working M1 CLI contract for planning purposes.

Initial capabilities in this milestone:

- define the public `zazz` command vocabulary and help surface for the initial CLI
- prerequisite and repo-container bootstrap/adoption checks performed by `zazz init`
- local `.zazz/` initialization
- define and persist the initial local DAG structure and node/edge metadata model
- facilitate the opinionated bare-repo plus sibling-worktree structure required by the project
- maintain a local integration worktree for `main` or `dev` as a clean, runnable origin-rooted source
- create a worktree node from either a local parent node or the configured remote integration branch
- generate or validate flat, meaningful worktree names
- automatically materialize configured local-only user files into a newly created worktree
- support a configurable source worktree and file-copy profile for local-only assets
- support verification that the new worktree is ready for actual development use
- list worktree nodes and show graph status
- show whether each node is current relative to its parent and the configured integration base
- inspect a node's parent/child relationships
- automatically refresh affected worktrees when the configured remote integration branch advances
- capture origin-refresh conflict artifacts in `.zazz/` with enough detail for agent or human handoff
- return structured conflict status responses for agent consumption
- support agent-assisted resolution for origin-refresh conflicts
- remove or clean up a managed worktree safely
- structured CLI output for agent consumption, such as JSON status responses

Likely deliverables:

- prerequisite detection inside `zazz init` and the Git interaction wrapper
- `.zazz/` DAG structure, operational ancestry model, and local state bootstrap
- repo-container and sibling-worktree bootstrap helpers
- local integration-worktree management
- local-user file sync profile and materialization logic
- worktree create/list/status/cleanup commands
- readiness verification for newly created worktrees
- graph inspection and JSON output contract
- integration-base freshness tracking in graph/status output
- remote-base refresh engine
- remote integration-branch drift detection
- conflict artifact capture and storage
- agent-facing conflict status contract
- agent-assisted resolution flow for origin-refresh conflicts

Suggested M1 deliverable slices

These are feature-level slices, not full SPECs. They are intended to be small enough that a human plus agent could plausibly complete each one in roughly a 12-hour implementation window, excluding later SPEC authoring time.

#### M1-D1: Local DAG/state bootstrap and repo-container validation

Focus:

- prerequisite detection inside `zazz init`
- repo-container adoption/bootstrap checks
- `.zazz/` structure
- node, edge, and operational ancestry model

Likely user-visible outcome:

- the CLI can tell whether a repo is ready and initialize trustworthy local orchestration state

#### M1-D2: Managed worktree creation and local-user file materialization

Focus:

- create from local parent or remote integration base
- opinionated sibling-worktree layout
- local integration-worktree convention
- local-only file and settings materialization
- readiness verification

Likely user-visible outcome:

- a new worktree can be created and used immediately without manual copying of ignored files

#### M1-D3: Worktree list/status and freshness inspection

Focus:

- list/status commands
- parent/child relationship inspection
- freshness signals relative to parent and integration base
- JSON output for agent consumption

Likely user-visible outcome:

- the user can see which worktrees are current, stale, blocked, or need refresh

#### M1-D4: Automated refresh and rebase across local and remote changes

Focus:

- remote integration-base refresh
- refresh ordering against the configured integration branch

Likely user-visible outcome:

- all managed worktrees in a repo can be refreshed against the shared origin branch without manual branch-by-branch rebasing

#### M1-D5: Conflict capture, persistence, and handoff

Focus:

- conflict classification
- repo-local conflict artifact storage
- machine-readable conflict output
- handoff contract for a human or agent resolver

Likely user-visible outcome:

- failed refreshes produce durable conflict records that can be handed to another agent instead of being lost in terminal output

#### M1-D6: Agent-assisted resolution for origin-refresh conflicts

Focus:

- agent-driven conflict resolution workflow
- CLI response contract for resolve attempts
- reconcile-success and reconcile-failure updates back into `.zazz/`

Likely user-visible outcome:

- when an origin refresh conflicts, an agent can be pointed at the conflict artifact and worktree to attempt resolution in a controlled way

#### M2-D1: PR lifecycle through the CLI

Focus:

- GitHub auth/adapters
- draft and ready-for-review PR creation
- PR state mirroring into local graph state

Likely user-visible outcome:

- managed worktrees can become reviewable PR units directly from the CLI

## Current State Summary

The feature is currently proposed only. No milestones have shipped yet.

## Planned Future Evolution

- PR lifecycle automation should become the next milestone after origin-refresh and conflict-handoff behavior is stable
- parent-to-child propagation after local PR merge should land in a later milestone after origin-refresh behavior is stable
- desktop visualization and review should build on top of this feature rather than replacing it
- future multi-machine or multi-user coordination may extend this feature, but that is out of MVP scope
- future AI-assisted PR review may complement the CLI, but human sign-off remains mandatory

## Risks, Constraints, and Non-Goals

Risks:

- Git edge cases may make propagation and recovery trickier than expected
- DAG semantics can become confusing without careful CLI language and state reporting
- AI-assisted conflict resolution must be observable and reviewable to maintain trust
- the distinction between intended work structure and current branch ancestry can become confusing if the CLI does not represent it clearly
- repo-local conflict artifacts must remain clean and discoverable or they will become their own source of chaos

Constraints:

- one-machine local orchestration for the initial release
- GitHub as first remote host
- flat sibling worktree naming and managed container layout
- branch and worktree names should be descriptive rather than relying on sequence numbers as dependency truth
- newly created worktrees should be made usable without forcing the user to manually recopy common local-only files every time

Non-goals:

- replacing the need for a human approver
- requiring the desktop app for core workflows
- distributed shared-state coordination in v1

## Open Questions

- should propagation run only on clean working states, or can the CLI safely manage dirty worktrees?
- how much AI context should be included by default during conflict resolution?
- what should the final public command namespace be?
- should intended work dependencies and current operational ancestry be stored as separate graph layers in `.zazz/`, or derived from a single richer model?
- should local-only files be copied, symlinked, or managed by a per-repo materialization strategy depending on file type?
- what should the exact schema and naming convention be for saved conflict artifacts under `.zazz/`?

## Deliverable Handoff Considerations

The next deliverables should likely start with:

- CLI foundation and local DAG metadata
- worktree creation and local-user file materialization
- remote-base refresh
- conflict artifact capture

Recommended first deliverable:

- implement the first slice of M1 around worktree bootstrap, local-user file materialization, and readiness verification so a new managed worktree is usable immediately

Core M1 deliverables that should be called out explicitly:

- local `.zazz/` DAG/state structure design and bootstrap
- local integration-worktree convention and management
- CLI worktree creation in the opinionated repo-container model
- automatic copying or materialization of untracked local files into new worktrees
- repo-local conflict artifact capture for failed origin refreshes

The desktop application should be specified as a separate feature that depends on this feature's core workflows rather than replacing them.

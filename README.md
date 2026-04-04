# zazzles CLI

`zazzles` is a worktree-native Git orchestration project for humans and agents.

## Project Status

**Work in progress (pre-1.0).**

- The CLI is usable for an initial M1 workflow.
- Command behavior and flags may evolve as additional workflows are implemented.
- The desktop application is planned but not implemented yet.

If you adopt `zaz` today, expect iterative changes while the command surface expands.

## What This Repository Provides Today

Current implemented CLI commands:

- `zaz init <repo-name> [--integration <branch>] [--json]`
- `zaz add <name> [--json]`

Current implemented behavior includes:

- managed repo bootstrap using a bare-repo + sibling-worktree layout
- repo-local state under `.zazz/`
- optional global integration-branch default from `~/.zazz/config.toml`
- untracked-file materialization based on repo-local manifest state
- both human and JSON output modes

Not implemented yet (planned direction):

- broader orchestration commands (`status`, `list`, `graph`, `sync`, `conflicts`, `resolve`, and PR lifecycle workflows)
- desktop graph/review UX

## Prerequisites

Install these tools before building or running the CLI:

- Rust stable toolchain with Cargo
- system `git`
- GitHub CLI (`gh`)

Notes:

- `zaz init` currently depends on `gh auth status` and `gh repo view`.
- You should authenticate GitHub CLI before running `zaz init`.

## Setup, Build, Install, and Run

### 1) Clone and enter the repository

```sh
git clone https://github.com/zazzcode/zazzles-cli.git
cd zazzles-cli
```

### 2) Build

Debug build:

```sh
cargo build -p zaz
```

Release build:

```sh
cargo build --release -p zaz
```

### 3) Run without installing

```sh
cargo run -p zaz -- --help
```

### 4) Install locally

```sh
cargo install --path apps/cli --force
```

Then run:

```sh
zaz --help
```

Compiled binaries are also available at:

- debug: `target/debug/zaz`
- release: `target/release/zaz`

## Quick Usage Example

From a parent directory where a managed repo root should be created:

```sh
zaz init zazz-skills --integration main
cd zazz-skills
zaz add smoke-test-1
```

JSON output examples:

```sh
zaz init zazz-skills --integration main --json
zaz add smoke-test-1 --json
```

## Important Current Constraints

- Pass a **bare repo name** to `zaz init` (example: `zazz-skills`), not a URL or `owner/repo`.
- Run `zaz init` from the parent directory where you want the new managed repo root created.
- Run `zaz add` from inside an already initialized managed repo root.
- The current M1 scope is greenfield bootstrap and add flow; broader adoption/migration workflows are planned separately.

## Developer Verification

Before opening PRs, run:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Repository Layout

- CLI entrypoint: `apps/cli`
- shared core orchestration logic: `crates/core`
- agent/repo workflow policy: `AGENTS.md`
- product and implementation docs: `docs/`

## Documentation

- proposal: `docs/proposals/worktree-native-pr-orchestrator.md`
- feature direction: `docs/features/worktree-orchestration-cli.md`
- companion feature docs index: `docs/features/worktree-orchestration-cli/README.md`
- operational worktree setup guidance: `docs/worktree-setup-instructions.md`

## Product Direction

Current direction is:

1. CLI-first stacked-branch workflow (MVP foundation)
2. richer sync/conflict/review orchestration
3. companion desktop experience
4. broader graph/DAG workflow support as the core model stabilizes

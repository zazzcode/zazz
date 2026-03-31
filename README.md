# zazzles

This repository is the starting point for `zazzles`, a Zazz-managed product focused on worktree-native stacked and graph-based PR workflows for both humans and agents.

The concept is to combine:

- a CLI that agents and power users can script, exposed publicly as `zaz`
- an installable desktop app for graph visualization, review, and conflict resolution
- worktree-native Git orchestration instead of virtual-branch indirection

## Current State

The repo is intentionally lightweight and currently contains:

- repo-level agent guidance in `AGENTS.md`
- imported Zazz framework skills in `.agents/skills/`
- proposal and framework docs under `docs/`
- initial standards and index files so future agent workflows can follow the Zazz document model cleanly

Product naming note:

- the product name is `zazzles`
- the company/framework name remains Zazz / zazzcode where applicable
- repo-local and user-global hidden state should continue to use `.zazz/` and `~/.zazz/`

## Build and Run

The CLI binary is named `zaz` and lives in `apps/cli`.

Prerequisites for local use:

- Rust toolchain with `cargo`
- system `git`
- GitHub CLI `gh`
- for the current M1 `zaz init` flow, a working `gh auth status`

Build the CLI:

```sh
cargo build -p zaz
```

Build the release binary:

```sh
cargo build --release -p zaz
```

Run the CLI directly through Cargo:

```sh
cargo run -p zaz -- --help
```

The built binaries are:

- debug: `target/debug/zaz`
- release: `target/release/zaz`

Current M1 usage:

```sh
# initialize from the parent directory where the repo root should be created
cargo run -p zaz -- init zazz-skills --integration main

# after init, run add from inside the managed repo root
cd zazz-skills
cargo run -p zaz -- add smoke-test-1
```

Recommended verification:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Docs Layout

```text
docs/
├── proposals/
├── features/
├── standards/
└── deliverables/
```

## Initial Direction

The current proposal recommends a phased build:

1. ship a CLI-first linear stacked-branch MVP
2. add GitHub-aware cascade sync and migration workflows
3. add a desktop graph/review experience
4. expand to DAG-style dependency graphs once the linear path is stable

See `docs/proposals/worktree-native-pr-orchestrator.md` for the detailed analysis, feature breakdown, sequence diagrams, and implementation recommendation, and `docs/features/worktree-orchestration-cli.md` for the current CLI feature direction.

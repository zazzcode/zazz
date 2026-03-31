# zazzles

`zazzles` is a worktree-native Git orchestration project for humans and agents. Its goal is to make isolated branch-per-worktree development practical for stacked or graph-shaped changes, with a scriptable CLI exposed as `zaz` and a future desktop app for graph visibility and review workflows.

The product direction combines:

- a CLI that agents and power users can script, exposed publicly as `zaz`
- an installable desktop app for graph visualization, review, and conflict resolution
- worktree-native Git orchestration instead of virtual-branch indirection

## What Exists Today

The repository now contains an initial Rust workspace and a working M1 CLI slice.

Implemented today:

- `zaz init <repo-name> [--integration <branch>]`
- `zaz add <name>`
- human-readable and `--json` output for both commands
- repo-local state in `.zazz/`
- user-global config lookup in `~/.zazz/config.toml`
- bootstrap and add behavior aligned to the current M1 deliverable docs

Current M1 behavior in plain language:

- `zaz init` bootstraps a new managed repo container from the parent directory where the repo root should be created
- `zaz init` keeps the repo root name identical to the repo name argument
- `zaz init` currently uses `gh repo view <repo-name>` plus `gh auth status` as part of its bootstrap contract
- `zaz add` creates a new worktree from the synced local integration worktree flow defined for M1
- `zaz add` materializes configured untracked files from repo-local `.zazz` state

Not implemented yet:

- the full planned CLI surface described in the feature docs, such as `status`, `list`, `graph`, `sync`, `conflicts`, `resolve`, account/profile commands, and PR lifecycle commands
- the desktop application
- native GitHub API replacement for the current M1 `gh` dependency

## Repository Layout

The repo is intentionally lightweight and currently contains:

- repo-level agent guidance in `AGENTS.md`
- imported Zazz framework skills in `.agents/skills/`
- proposal and framework docs under `docs/`
- a Rust workspace with the CLI in `apps/cli` and shared orchestration logic in `crates/core`

Product naming note:

- the product name is `zazzles`
- the company/framework name remains Zazz / zazzcode where applicable
- repo-local and user-global hidden state should continue to use `.zazz/` and `~/.zazz/`

## Compile, Install, and Run

The CLI binary is named `zaz` and lives in `apps/cli`.

Prerequisites for local use:

- Rust toolchain with `cargo`
- system `git`
- GitHub CLI `gh`
- for the current M1 `zaz init` flow, a working `gh auth status`

Compile the debug build:

```sh
cargo build -p zaz
```

Compile the release build:

```sh
cargo build --release -p zaz
```

Install the CLI into Cargo's bin directory:

```sh
cargo install --path apps/cli --force
```

If Cargo's bin directory is on your `PATH`, you can then run:

```sh
zaz --help
```

Run the CLI through Cargo without installing:

```sh
cargo run -p zaz -- --help
```

Run the compiled binaries directly:

- debug: `target/debug/zaz`
- release: `target/release/zaz`

## Command Overview

Current implemented commands:

```sh
zaz init <repo-name> [--integration <branch>] [--json]
zaz add <name> [--json]
```

Examples:

```sh
# initialize from the parent directory where the repo root should be created
cargo run -p zaz -- init zazz-skills --integration main

# after init, run add from inside the managed repo root
cd zazz-skills
cargo run -p zaz -- add smoke-test-1
```

The same commands work with the installed binary:

```sh
zaz init zazz-skills --integration main
cd zazz-skills
zaz add smoke-test-1
```

JSON mode examples:

```sh
zaz init zazz-skills --integration main --json
zaz add smoke-test-1 --json
```

## Typical M1 Usage Flow

1. From a parent directory, run `zaz init <repo-name> [--integration <branch>]`.
2. Let `zaz init` create the managed repo root, `.bare/`, integration worktree, and `.zazz/` state.
3. Change into the new repo root.
4. Run `zaz add <name>` to create a new sibling worktree and branch from the synced local integration worktree.
5. Work inside the new worktree directory that was created under the managed repo root.

Important M1 constraints:

- pass a bare repo name such as `zazz-skills`, not a clone URL
- run `zaz init` from the parent directory where the repo root should be created
- run `zaz add` from inside an already initialized managed repo root
- M1 is greenfield bootstrap only; existing compatible `.bare/` layouts are described in feature docs as future adoption work, not current CLI behavior

Recommended verification:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Docs

Primary docs:

- proposal: `docs/proposals/worktree-native-pr-orchestrator.md`
- main CLI feature doc: `docs/features/worktree-orchestration-cli.md`
- CLI companion docs index: `docs/features/worktree-orchestration-cli/README.md`
- worktree setup guidance: `docs/worktree-setup-instructions.md`

Docs layout:

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

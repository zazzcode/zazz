# Rust CLI Development Best Practices

## Purpose

This standard defines how zazzles should build and evolve its Rust CLI and the Rust modules and crates that exist specifically to support that CLI.

Use this guidance for:

- CLI command design
- shared core orchestration logic
- filesystem and process integration
- testing and review expectations for Rust code

## When To Apply This Standard

Apply this standard when a change does any of the following:

- adds or changes Rust crates, modules, or Cargo manifests
- defines or revises CLI command contracts, flags, exit behavior, or JSON output
- introduces `.zazz/` state files, config formats, or serialization logic
- adds Git, GitHub, filesystem, or process-invocation adapters in Rust
- reviews whether a Rust implementation is ready for agent use or human maintenance

This standard should also guide deliverable specs that tell agents how to build Rust CLI behavior, even before much Rust code exists in the repo.

Do not use this standard as the primary guide for future web apps, desktop UI implementation details, or backend services. Those should get their own focused standards if and when they are introduced.

## Architectural Boundaries

- Keep `apps/cli` thin. Argument parsing, output formatting, and exit-code mapping belong there.
- Keep business logic, repo-state handling, Git/GitHub adapters, and reusable orchestration code that serves the CLI in `crates/`.
- Prefer adding modules inside an existing crate before introducing a new crate. Add a new crate only when it creates a clear ownership boundary or reuse benefit.
- The CLI should call shared library code through explicit request/response types rather than embedding business rules directly in `main.rs`.
- Cargo workspaces should remain the top-level composition mechanism for shared package metadata, lockfile management, and workspace-wide commands.

## File and Module Size

- No source file should grow beyond roughly 500 lines.
- If a file approaches that limit, split it by responsibility before adding more code.
- Prefer decomposition into modules such as `commands`, `state`, `git`, `github`, `materialize`, `output`, and `tests` rather than keeping large mixed-purpose files.
- If a command has both human and JSON rendering, keep rendering code separate from orchestration code.

## CLI Design

- Use a stable parser crate such as `clap` with derive-based structs for commands, flags, and arguments.
- Keep subcommands explicit and strongly typed.
- Support machine-readable output with structured Rust types serialized through `serde`.
- Do not build JSON responses by hand with string concatenation.
- Human-readable output may be more descriptive, but JSON output should remain stable and minimal.
- If a command supports `--json`, define the response struct in shared code so tests can validate it directly.
- Put validation in typed argument parsing or explicit request validation when possible so failures are early and consistent.

## Error Handling

- Use typed domain errors in shared crates.
- Map domain errors to user-facing messages and exit behavior at the CLI boundary.
- Do not hide actionable stderr from failed `git` or `gh` commands; capture it and surface concise remediation guidance.
- Avoid panics for expected operational failures such as missing tools, invalid repo layout, auth failures, or filesystem conflicts.
- Keep failure categories explicit so JSON output and tests can distinguish prerequisite, auth, Git, config, and materialization failures.

## Process Execution

- Invoke external tools with `std::process::Command`; do not shell out through `sh -c` or `zsh -c` from Rust code.
- Centralize `git` and `gh` process execution behind small adapter functions or types instead of scattering raw command calls across the codebase.
- Capture stdout, stderr, exit status, and the invoked arguments for debugging and test assertions.
- Validate paths and arguments before invoking external commands.

## Filesystem and State

- Represent `.zazz/` state with typed structs serialized through `serde`.
- Prefer atomic file writes for repo-local state files so failed writes do not leave truncated JSON or TOML behind.
- Resolve all relative paths from an explicit repo root or worktree root, never from an ambient current directory hidden deep in the call stack.
- Keep machine-specific configuration explicit. Do not hard-code one developer's filesystem paths into reusable command logic.
- Validate required state before mutating the filesystem.
- Use `serde` derives and attributes intentionally; prefer explicit field names and validation-friendly structs over ad hoc maps when the schema is part of a command contract.

## Dependency Guidance

- Prefer small, well-maintained crates with clear value.
- Good default choices for this repo include:
  - `clap` for CLI parsing
  - `serde` and `serde_json` for JSON state and output
  - `toml` for config files
  - `thiserror` for typed errors
  - `tempfile` for tests involving repos and worktrees
  - `assert_cmd` and `predicates` for command-level tests
- Add async runtime dependencies only if a concrete requirement needs them. The M1 CLI flow should stay synchronous unless proven otherwise.
- Prefer workspace-managed dependency versions where shared crates need the same libraries.

## Testing Expectations

- Put most logic under test in shared crates, not only through the binary entrypoint.
- Add command-level integration tests for each user-facing command and `--json` mode.
- Use fixture repos or temporary test repos for Git-heavy behavior.
- Test both success and failure paths, especially around partial-state prevention.
- Assert on exit status, filesystem effects, and structured output contracts.
- Prefer targeted tests over broad snapshots that hide behavior changes.
- Keep unit tests close to implementation modules and integration tests focused on public behavior across module boundaries.

## Quality Gates

- Format with `cargo fmt`.
- Lint with `cargo clippy`.
- Run tests with `cargo test`.
- Prefer workspace-scoped verification for shared changes, for example `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- If we later want centrally enforced Rust lints, prefer workspace-level lint configuration rather than inconsistent crate-by-crate drift.

## Review Expectations

- New commands should show a clear separation between parsing, orchestration, and rendering.
- Shared types should be named around domain concepts rather than transport details.
- Reviewer-visible behavior should be traceable from spec requirements to tests.
- If a change pushes a file past the size guidance, treat decomposition as part of the work, not a follow-up nice-to-have.

## M1 CLI Guidance For This Repo

- Keep `apps/cli/src/main.rs` as a small entrypoint that wires command parsing to shared handlers.
- Place `.zazz/` schema types and initialization logic in shared code under `crates/core`.
- Isolate Git interactions behind a dedicated adapter module so later desktop and daemon surfaces can reuse the same orchestration layer.
- Treat human output and JSON output as separate rendering concerns.

## References

These sources informed the standard. Some repo-specific rules above, such as the 500-line decomposition rule and the exact `apps/cli` vs `crates/core` split, are local standards built on top of these sources.

- Rust Book, command-line project guidance: <https://doc.rust-lang.org/book/ch12-00-an-io-project.html>
- Rust Book, test organization: <https://doc.rust-lang.org/book/ch11-03-test-organization.html>
- Cargo Book, workspaces: <https://doc.rust-lang.org/cargo/reference/workspaces.html>
- Cargo Book, `cargo fmt`: <https://doc.rust-lang.org/cargo/commands/cargo-fmt.html>
- Cargo Book, `cargo clippy`: <https://doc.rust-lang.org/cargo/commands/cargo-clippy.html>
- Standard library, `std::process::Command`: <https://doc.rust-lang.org/std/process/struct.Command.html>
- clap derive tutorial: <https://docs.rs/clap/latest/clap/_derive/_tutorial/>
- Serde overview and derive usage: <https://serde.rs/> and <https://serde.rs/derive.html>
- Rust API Guidelines: <https://rust-lang.github.io/api-guidelines/>

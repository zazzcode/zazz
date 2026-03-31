# App

This directory is reserved for the future zazzles app host and UI application shell.

The current Rust workspace starts with the CLI in `apps/cli` and shared libraries in `crates/`.

Naming direction:

- use `client` as the repo directory name for the installable local application host
- keep product language centered on the zazzles app and UI rather than the older `desktop` label
- this is the installable local application surface for zazzles, regardless of whether the user is on a laptop or desktop machine

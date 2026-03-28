# Repo Foundations

## Purpose

This document captures the initial repo-wide standards for the worktree-native PR orchestration product while the codebase is still being established.

## Product Direction

- Build a CLI that is friendly to both humans and agents.
- Treat the CLI as the primary execution surface.
- Treat the desktop application as a companion experience built on the same core engine, not as a separate product.

## Architecture Direction

- Prefer a shared core domain and orchestration engine that can be consumed by both the CLI and the desktop application.
- For the first implementation, prefer calling the system `git` executable over re-implementing complex Git behavior in-process.
- Start with GitHub as the first supported remote host and defer multi-host support until the core workflow is stable.

## Workflow Direction

- Optimize for one branch per worktree and one active worktree per unit of work.
- Model stack or graph relationships as explicit metadata rather than filesystem hierarchy.
- Keep durable planning and product knowledge in tracked docs under `docs/`.

## Interface Direction

- Every important CLI command should support machine-readable output.
- JSON output is required for any command that agents may need to parse.
- Long-running sync, cascade, or review state should eventually be streamable to a UI or daemon subscriber.

## UI Direction

- A Rust-backed desktop application is encouraged, but the initial recommendation is Tauri plus a web-based UI layer for the graph, diff, and review surfaces.
- Do not force a pure-Rust desktop UI for v1 if it slows down graph visualization, diff rendering, or review UX quality.

## Delivery Direction

- MVP should validate the sequential stacked-branch workflow before the full DAG workflow.
- Conflict handling should start with safe Git-native rebases and add semantic migration as the escalation path for hard cases.

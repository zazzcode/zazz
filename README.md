# Worktree-Native PR Orchestrator

This repository is the starting point for a product focused on worktree-native stacked and graph-based PR workflows for both humans and agents.

The concept is to combine:

- a CLI that agents and power users can script
- an installable desktop app for graph visualization, review, and conflict resolution
- worktree-native Git orchestration instead of virtual-branch indirection

## Current State

The repo is intentionally lightweight and currently contains:

- repo-level agent guidance in `AGENTS.md`
- imported Zazz framework skills in `.agents/skills/`
- proposal and framework docs under `docs/`
- initial standards and index files so future agent workflows can follow the Zazz document model cleanly

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

See `docs/proposals/worktree-native-pr-orchestrator.md` for the detailed analysis, feature breakdown, sequence diagrams, and implementation recommendation.

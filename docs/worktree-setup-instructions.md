# Zazzles Worktree Setup Instructions

This guide is for agents and humans working in the `zazzles` repository using the bare-repo plus sibling-worktree layout.

This is an operational companion document. Durable product expectations should still live in feature docs and deliverable specs. When this guide and a deliverable spec disagree, the deliverable spec should be treated as the implementation contract for that deliverable.

This guide is also the manual source document for the workflow that the CLI should automate. In plain terms:

- `zaz init <repo-name> [--integration <branch>]` should automate repo root creation, repo validation, `.zazz/` initialization, and shared setup checks.
- `zaz add` should automate preflight sync, new-worktree creation, untracked-file materialization, and repo-specific post-create setup.

## Why This Structure

This repo uses an opinionated bare-repo plus sibling-worktree layout on purpose.

Why we prefer it:

- all Git objects live once in `.bare/`, so additional worktrees duplicate working files instead of the full repository history
- the repo root shows the whole active workspace in one place: `.bare/`, `.zazz/`, the integration worktree, and sibling feature worktrees
- branch directory names can match branch names, which makes it easier for humans and agents to see what is active
- the repo root becomes a stable place for repo-local orchestration state such as `.zazz/` and shared ignore rules in `.bare/info/exclude`
- the integration worktree can remain a clean, always-runnable reference point for creating new worktrees

This rationale is influenced by:

- Ahmed El Gabri, [Git Worktrees Done Right](https://gabri.me/blog/git-worktrees-done-right), especially the visibility and isolation benefits of the bare-repo pattern
- the official [git worktree documentation](https://git-scm.com/docs/git-worktree.html), which defines the underlying linked-worktree model this layout builds on

Important repo-specific note:

- Ahmed El Gabri's article also describes keeping some personal local files at the repo root outside any worktree
- this repo's current M1 CLI contract is different: untracked files that should be copied to future worktrees are treated as integration-worktree content recorded in `.zazz/untracked-files.json`
- that difference is intentional and the deliverable spec remains the source of truth for M1 automation behavior

## Current Scope

For the current M1 deliverable, the only required automated creation flow is:

- initialize the repo root with `zaz init <repo-name> [--integration <branch>]` from the parent directory where the repo root should be created
- create a new branch and sibling worktree from the synced local integration worktree with `zaz add <name>`

This guide includes one broader manual pattern for adopting an already-existing remote branch, but that is not part of the current `init-add-worktree` deliverable contract.

## Command Mapping

Use this mapping when deciding whether behavior belongs in `zaz init`, `zaz add`, or a later command.

### `zaz init`

`zaz init` is the repo-readiness and initialization command. It should own:

- validating the requested repo name
- validating that the current working directory is the parent directory where the repo root should be created
- validating required local tools such as `git` and `gh`
- validating GitHub access for the named repo
- creating `<cwd>/<repo-name>/` as the repo root
- creating `<cwd>/<repo-name>/.bare/` as the bare Git directory
- creating `<cwd>/<repo-name>/<integration-branch>/` as the integration worktree
- creating `<cwd>/<repo-name>/.zazz/` only after all required checks pass
- validating or configuring `.bare/info/exclude`
- seeding repo-local configuration needed by later commands, including untracked-file materialization config

### `zaz add`

`zaz add` is the new-worktree creation command. For the current M1 flow, it should own:

- fetching remote state
- fast-forwarding the local integration worktree
- creating a new sibling worktree from the synced local integration worktree
- copying configured untracked files into the new worktree
- syncing repo-specific local skills or equivalent local setup
- reporting whether the new worktree is ready for normal development

## Quick-Start Checklist

To bootstrap a repo root and then create the first feature worktree:

1. Run `zaz init <repo-name>` from the parent directory where the repo root should be created.
2. Preflight: fetch latest and sync the integration worktree.
3. Create the worktree from the synced local integration worktree.
4. Ensure shared excludes are configured in `.bare/info/exclude`.
5. Materialize untracked files from the configured source worktree when needed.
6. Sync local Zazz skills if this repo is using that local workflow.
7. Bootstrap dependencies when the repo defines them.
8. Run the verification checklist.

## Working Model

- The repo root directory name must match the repo name passed to `zaz init`.
- Repository root is the top-level directory holding `.bare/`, `.zazz/`, and sibling worktrees.
- Bare Git directory lives at `.bare/`.
- Repo-local orchestration state lives at `.zazz/`.
- Integration worktree is the resolved branch selected during `zaz init`.
- Feature worktrees use one sibling directory per feature or deliverable branch.
- Do not merge feature branches locally into the integration branch; integration should happen through PRs.

### Git Command Style

The repo root is not a normal checkout. Use `--git-dir` for repo-level Git operations and `-C` for worktree-specific operations.

```bash
WT_ROOT=/path/to/zazzles

git --git-dir="$WT_ROOT/.bare" <git-command>
git -C "$WT_ROOT/<worktree-name>" <git-command>
```

## 0. Initialize The Repo Root

Run `zaz init <repo-name> [--integration <branch>]` from the parent directory where you want the managed repo root to be created.

Rules:

- The first positional argument is the repo name.
- `--integration <branch>` and `-i <branch>` are optional.
- Integration branch resolution order is: explicit flag, user-global config default, fallback `main`.
- The user-global config file is `~/.zazz/config.toml`, and this workflow reads top-level `integration_branch` from it when present.
- The repo name and the directory name must match exactly.
- M1 does not support a separate custom directory argument.
- The command should create a directory named `<repo-name>/` in the current working directory.
- The command should create the opinionated layout inside that directory: `.bare/`, the resolved integration worktree, and `.zazz/`.
- M1 supports fresh bootstrap only. It does not adopt or normalize an already-existing local repo layout.

Target layout after a successful init:

```text
<parent>/
└── <repo-name>/
    ├── .bare/
    ├── .zazz/
    └── <integration-branch>/
```

## 1. Preflight

Fetch the latest remote state and fast-forward the local integration worktree before branching:

```bash
WT_ROOT=/path/to/zazzles

git --git-dir="$WT_ROOT/.bare" fetch --prune origin
git -C "$WT_ROOT/<integration-branch>" pull --ff-only
git --git-dir="$WT_ROOT/.bare" worktree list
```

In this repo, the resolved integration worktree is the canonical local source for new M1 worktree creation. The standard workflow is:

1. Fetch from `origin`.
2. Fast-forward the integration worktree.
3. Create the new worktree from that synced local integration worktree.

Using the synced local integration worktree as the source avoids creating the feature branch directly from a remote-tracking ref and keeps the new branch free to set its own upstream later.

## 2. Create The Worktree

Choose the worktree creation mode based on whether the target branch already exists on `origin`.

- If the branch does not exist on `origin`, create a new feature branch from the synced local integration worktree.
- If the branch already exists on `origin`, create a local tracking worktree from `origin/<branch>`.

For the current `init-add-worktree` deliverable, only the first path is in scope for automation. The second path remains a documented manual workflow for later CLI support.

### Detect The Correct Use Case

```bash
BRANCH=<branch-name>
WT_ROOT=/path/to/zazzles

if git --git-dir="$WT_ROOT/.bare" branch -r --list "origin/$BRANCH" | grep -q .; then
  echo "Use Case B: existing remote branch"
else
  echo "Use Case A: new branch from local integration worktree"
fi
```

### Use Case A: New Feature Branch From Local Integration Worktree

```bash
NEW_BRANCH=<your-feature-branch>
INTEGRATION_BRANCH=<integration-branch>
WT_ROOT=/path/to/zazzles

git --git-dir="$WT_ROOT/.bare" worktree add "$WT_ROOT/$NEW_BRANCH" -b "$NEW_BRANCH" "$INTEGRATION_BRANCH"
```

For the current `init-add-worktree` deliverable, `zaz add` only automates creation from the configured integration worktree.

### Use Case B: Existing Remote Branch

This is a valid manual repo workflow, but it is out of scope for the current M1 `zaz add` implementation.

```bash
BRANCH=<existing-remote-branch>
WT_ROOT=/path/to/zazzles

git --git-dir="$WT_ROOT/.bare" worktree add "$WT_ROOT/$BRANCH" --track -b "$BRANCH" "origin/$BRANCH"
```

### Validate The Result

```bash
TARGET_BRANCH=<branch-name>
WT_ROOT=/path/to/zazzles

git -C "$WT_ROOT/$TARGET_BRANCH" status --short --branch
git -C "$WT_ROOT/$TARGET_BRANCH" log --oneline -1
git -C "$WT_ROOT/$TARGET_BRANCH" rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null || echo "(no upstream yet)"
```

To track remote immediately for a new local branch:

```bash
NEW_BRANCH=<your-feature-branch>
WT_ROOT=/path/to/zazzles

git -C "$WT_ROOT/$NEW_BRANCH" push -u origin "$NEW_BRANCH"
```

Creating the worktree only checks out Git-tracked files. The worktree is not ready until the remaining setup steps are complete.

## 3. Shared Excludes In `.bare/info/exclude`

Some repo-local execution artifacts should be ignored across all worktrees under this repo root.

The shared exclude file for this repo lives at:

```bash
.bare/info/exclude
```

For this repo, the shared exclude file already exists as part of local setup. A normal new worktree should not need extra manual changes here unless repo-level ignore rules have changed.

`zaz init` should own this behavior for the M1 CLI path, including validation that the shared exclude file exists and any required repo-local patterns are present.

## 4. Copy Untracked Files

Untracked files are not tracked by Git. Copy them from the configured source worktree, which is the local integration worktree for this repo unless a deliverable spec says otherwise.

For the `zazzles` repo, the exact untracked-file materialization profile should evolve alongside the source-of-truth deliverable specs for `zaz init` and `zaz add`.

### Current Direction

`zaz init` should create or seed the repo-local manifest for untracked-file materialization, and `zaz add` should follow that saved manifest when creating a new worktree.

The CLI should treat untracked-file materialization as repo-specific configuration rather than hard-coded app assumptions.

Examples of assets that may eventually be materialized:

- untracked environment files
- user-specific tool settings
- generated local support files
- repo-local execution documents or scratch artifacts when the repo chooses that pattern

### Copy Placeholder

```bash
SOURCE_WT=/path/to/zazzles/<integration-branch>
TARGET_WT=/path/to/zazzles/<new-branch>
MANIFEST=.zazz/untracked-files.json

echo "TODO: zaz should read $MANIFEST and materialize configured untracked files from $SOURCE_WT into $TARGET_WT"
```

The important rule is that `zaz init` should seed the manifest from the current untracked files in the integration worktree, and `zaz add` should later materialize only the paths explicitly recorded in that saved repo-local state. `zaz add` should not infer new copy targets from arbitrary current untracked files at add time.

When seeding that manifest, prefer one directory entry over many descendant file entries, default seeded entries to `required = false`, and write the saved entries in stable lexicographic order.

## 5. Sync Local Zazz Skills

This project currently uses local skills from a sibling `zazz-skills` checkout copied or synced into each worktree.

For the current M1 deliverable, this is treated as repo-specific post-create setup that should happen after worktree creation and after untracked-file materialization.

The current opinionated source location should be treated as relative to the repo root:

```text
../zazz-skills/.agents/skills
```

### Exclude Setup

```bash
WT_ROOT=/path/to/zazzles
SKILLS_SRC=/path/to/zazz-skills/.agents/skills
EXCLUDE_FILE="$WT_ROOT/.bare/info/exclude"

for src_dir in "$SKILLS_SRC"/*; do
  [ -d "$src_dir" ] || continue
  name=$(basename "$src_dir")
  pattern=".claude/skills/$name/"
  grep -Fx "$pattern" "$EXCLUDE_FILE" >/dev/null 2>&1 || printf "%s\n" "$pattern" >> "$EXCLUDE_FILE"
done
```

### Per-Worktree Sync

```bash
TARGET_WT=/path/to/zazzles/<new-branch>
SKILLS_SRC=/path/to/zazz-skills/.agents/skills

mkdir -p "$TARGET_WT/.claude/skills"
for src_dir in "$SKILLS_SRC"/*; do
  [ -d "$src_dir" ] || continue
  name=$(basename "$src_dir")
  mkdir -p "$TARGET_WT/.claude/skills/$name"
  rsync -a --delete --exclude '.git' "$src_dir/" "$TARGET_WT/.claude/skills/$name/"
done
```

Rules:

- Sync skills into the newly created target worktree only.
- Do not backfill older worktrees unless explicitly requested.
- If this repo later replaces the local skills-sync workflow, update this guide and the relevant deliverable specs together.

## 6. Bootstrap Dependencies

This section remains a placeholder until the repo has real bootstrap commands.

Today, the important setup outcome is structural rather than runtime-specific:

- the worktree exists
- `.zazz/` exists and is valid
- shared excludes are configured
- any repo-specific untracked files have been materialized

When the repo has real bootstrap commands, update this section and decide whether they belong inside `zaz add`, a separate bootstrap command, or both.

## 7. Verification Checklist

Run after completing all previous steps. This is the practical definition of "ready for development" for a freshly created worktree:

```bash
WT=/path/to/zazzles/<new-branch>

git -C "$WT" status --short --branch
git -C "$WT" rev-parse --abbrev-ref HEAD
git -C "$WT" rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null || echo "(no upstream yet)"

for f in "$WT"/.claude/skills/*/SKILL.md; do
  [ -e "$f" ] || continue
  rel="${f#$WT/}"
  git -C "$WT" check-ignore -q "$rel" && echo "OK  $rel (ignored)" || echo "WARN not ignored: $rel"
done

[ -f "$WT/../.bare/info/exclude" ] && echo "OK  .bare/info/exclude" || echo "MISS .bare/info/exclude"
[ -d "$WT/../.zazz" ] && echo "OK  .zazz/" || echo "MISS .zazz/"
```

All items should show `OK`. Resolve any `MISS` or `WARN` before starting development.

### Smoke-Test Reference

For the current `init-add-worktree` deliverable, the canonical workspace-local smoke-test layout lives under:

```text
.tmp/manual-smoke/
└── zazz-skills/
    ├── .bare/
    ├── .zazz/
    ├── main/
    └── smoke-test-1/
```

When running that smoke test:

- initialize the repo root under `.tmp/manual-smoke/`
- create synthetic root-level untracked files in the integration worktree such as `.env.local` and `.smoke-notes.txt`
- create a synthetic untracked directory such as `zaz-smoke-untracked/` with nested contents
- ensure those paths are listed in `.zazz/untracked-files.json`
- run `zaz add smoke-test-1` and verify the new sibling worktree receives those files

The full smoke-test contract for the overnight implementation run is defined in:

- `docs/deliverables/init-add-worktree-SPEC.md`
- `docs/deliverables/init-add-worktree-PLAN.md`

## 8. Day-To-Day Usage

⚠️ Agent Directive ⚠️ MUST NOT: Make code, docs, or config edits directly in the integration worktree (typically `main`).
⚠️ Agent Directive ⚠️ MUST: Perform implementation and documentation edits in a non-integration feature worktree, then merge via PR.

- Do coding in feature worktrees only.
- Keep the integration worktree synced from remote and clean.
- Open PRs from feature branches into the configured integration branch.
- Avoid local merges into the integration branch; use the PR merge flow as the integration mechanism.

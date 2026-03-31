use crate::commands::AddRequest;
use crate::config::{RepoConfig, read_repo_config};
use crate::errors::{CommandFailure, FailureCategory, OperationFailure};
use crate::git::GitClient;
use crate::materialize::materialize;
use crate::output::{AddCommandPayload, CheckStatus};
use crate::process::ToolNames;
use crate::setup::{ensure_shared_excludes, sync_local_skills};
use crate::state::{
    GraphNode, ManagedGraph, ManagedWorktreeRecord, WorktreeKind, WorktreesState, read_graph_state,
    read_manifest, read_worktrees_state, write_graph_state, write_worktrees_state,
};
use std::fs;
use std::path::{Path, PathBuf};

pub fn execute(
    request: AddRequest,
) -> Result<AddCommandPayload, Box<CommandFailure<AddCommandPayload>>> {
    let mut state = AddExecutionState::new(&request);
    if let Err(failure) = validate_branch_name(&state.branch) {
        return Err(state.fail(failure));
    }

    let repo_config = match read_repo_config(&state.repo_root) {
        Ok(config) => {
            state.repo_name = Some(config.repo_name.clone());
            state.integration_branch = config.integration_branch.clone();
            state.integration_worktree_path =
                Some(state.repo_root.join(&config.integration_worktree));
            config
        }
        Err(failure) => return Err(state.fail(failure)),
    };
    state.worktree_path = Some(state.repo_root.join(&state.branch));

    let manifest = match read_manifest(&state.repo_root) {
        Ok(manifest) => manifest,
        Err(failure) => return Err(state.fail(failure)),
    };
    let current_graph = match read_graph_state(&state.repo_root) {
        Ok(graph) => graph,
        Err(failure) => return Err(state.fail(failure)),
    };
    let current_worktrees = match read_worktrees_state(&state.repo_root) {
        Ok(worktrees) => worktrees,
        Err(failure) => return Err(state.fail(failure)),
    };

    let target_worktree = state
        .worktree_path
        .clone()
        .expect("worktree path should already be set");
    if target_worktree.exists() {
        return Err(state.fail(OperationFailure::new(
            FailureCategory::Filesystem,
            format!("target worktree directory already exists: {}", target_worktree.display()),
            "Choose a different worktree name or remove the existing sibling directory before retrying `zaz add`.",
        )));
    }

    let tools = ToolNames::from_env();
    let git = GitClient::new(&tools);
    if let Err(failure) = git.ensure_available() {
        state.checks.push(CheckStatus {
            name: "git available".into(),
            ok: false,
            detail: failure.message.clone(),
        });
        return Err(state.fail(failure));
    }
    state.checks.push(CheckStatus {
        name: "git available".into(),
        ok: true,
        detail: "git is callable".into(),
    });

    let bare_dir = state.repo_root.join(".bare");
    match git.local_branch_exists(&bare_dir, &state.branch) {
        Ok(true) => {
            return Err(state.fail(OperationFailure::new(
                FailureCategory::Git,
                format!("branch `{}` already exists locally", state.branch),
                "Choose a different worktree name or delete the existing local branch before retrying `zaz add`.",
            )))
        }
        Ok(false) => {}
        Err(failure) => return Err(state.fail(failure)),
    }

    if let Err(failure) = git.fetch_prune_origin(&bare_dir) {
        state.checks.push(CheckStatus {
            name: "fetch origin".into(),
            ok: false,
            detail: failure.message.clone(),
        });
        return Err(state.fail(failure));
    }
    state.checks.push(CheckStatus {
        name: "fetch origin".into(),
        ok: true,
        detail: "fetched origin with prune".into(),
    });

    let integration_worktree = state
        .integration_worktree_path
        .clone()
        .expect("integration worktree path should already be set");
    if let Err(failure) = git.pull_ff_only(&integration_worktree) {
        state.checks.push(CheckStatus {
            name: "pull integration".into(),
            ok: false,
            detail: failure.message.clone(),
        });
        return Err(state.fail(failure));
    }
    state.checks.push(CheckStatus {
        name: "pull integration".into(),
        ok: true,
        detail: format!("fast-forwarded {}", state.integration_branch),
    });

    if let Err(failure) = git.add_worktree_new_branch(
        &bare_dir,
        &target_worktree,
        &state.branch,
        &state.integration_branch,
    ) {
        state.checks.push(CheckStatus {
            name: "create worktree".into(),
            ok: false,
            detail: failure.message.clone(),
        });
        return Err(state.fail(failure));
    }
    state.created_worktree = true;
    state.checks.push(CheckStatus {
        name: "create worktree".into(),
        ok: true,
        detail: format!("created {}", target_worktree.display()),
    });

    if let Err(failure) = ensure_shared_excludes(&state.repo_root) {
        cleanup_partial_worktree(&mut state, &git);
        return Err(state.fail(failure));
    }

    let materialization = match materialize(&integration_worktree, &target_worktree, &manifest) {
        Ok(report) => report,
        Err(failure) => {
            cleanup_partial_worktree(&mut state, &git);
            return Err(state.fail(failure));
        }
    };
    state.materialized = !materialization.copied.is_empty();
    state.checks.push(CheckStatus {
        name: "materialize untracked".into(),
        ok: true,
        detail: format!(
            "copied {} manifest entries; skipped {} optional missing entries",
            materialization.copied.len(),
            materialization.missing_optional.len()
        ),
    });

    let skills_sync = match sync_local_skills(&state.repo_root, &repo_config, &target_worktree) {
        Ok(report) => report,
        Err(failure) => {
            cleanup_partial_worktree(&mut state, &git);
            return Err(state.fail(failure));
        }
    };
    state.checks.push(CheckStatus {
        name: "sync skills".into(),
        ok: true,
        detail: if skills_sync.skipped_missing_source {
            format!(
                "skipped missing skills source {}",
                repo_config.skills_sync_source
            )
        } else {
            format!("synced {} local skills", skills_sync.copied_skills.len())
        },
    });

    if let Err(failure) =
        persist_registration(&state, &current_graph, &current_worktrees, &repo_config)
    {
        cleanup_partial_worktree(&mut state, &git);
        return Err(state.fail(failure));
    }
    state.checks.push(CheckStatus {
        name: "register state".into(),
        ok: true,
        detail: "updated .zazz graph and worktree registry".into(),
    });

    match git.current_branch(&target_worktree) {
        Ok(current_branch) if current_branch == state.branch => {
            state.checks.push(CheckStatus {
                name: "readiness".into(),
                ok: true,
                detail: format!("worktree is ready on branch {}", state.branch),
            });
            Ok(state.success())
        }
        Ok(current_branch) => {
            cleanup_partial_worktree(&mut state, &git);
            Err(state.fail(OperationFailure::new(
                FailureCategory::Git,
                format!(
                    "new worktree is on `{current_branch}` instead of `{}`",
                    state.branch
                ),
                "Inspect the partially created worktree state and retry `zaz add`.",
            )))
        }
        Err(failure) => {
            cleanup_partial_worktree(&mut state, &git);
            Err(state.fail(failure))
        }
    }
}

fn validate_branch_name(branch: &str) -> Result<(), OperationFailure> {
    let allowed = branch
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'));
    if branch.is_empty() || !allowed || branch.contains('/') || branch.contains(':') {
        return Err(OperationFailure::new(
            FailureCategory::Config,
            format!("`{branch}` is not a valid M1 worktree name"),
            "Use a flat branch name with letters, numbers, dots, underscores, or hyphens for `zaz add`.",
        ));
    }
    Ok(())
}

fn persist_registration(
    state: &AddExecutionState,
    current_graph: &ManagedGraph,
    current_worktrees: &WorktreesState,
    config: &RepoConfig,
) -> Result<(), OperationFailure> {
    let target_path = display_path(
        state
            .worktree_path
            .as_ref()
            .expect("worktree path should already be set"),
    );
    let mut next_graph = current_graph.clone();
    next_graph.nodes.push(GraphNode {
        branch: state.branch.clone(),
        worktree: state.branch.clone(),
        path: target_path.clone(),
        parent_branch: Some(config.integration_branch.clone()),
        kind: WorktreeKind::Feature,
    });

    let mut next_worktrees = current_worktrees.clone();
    next_worktrees.items.push(ManagedWorktreeRecord {
        branch: state.branch.clone(),
        worktree: state.branch.clone(),
        path: target_path,
        kind: WorktreeKind::Feature,
    });

    write_graph_state(&state.repo_root, &next_graph)?;
    if let Err(failure) = write_worktrees_state(&state.repo_root, &next_worktrees) {
        let _ = write_graph_state(&state.repo_root, current_graph);
        return Err(failure);
    }
    Ok(())
}

fn cleanup_partial_worktree(state: &mut AddExecutionState, git: &GitClient) {
    if !state.created_worktree {
        return;
    }
    let bare_dir = state.repo_root.join(".bare");
    if let Some(target) = state.worktree_path.clone() {
        if let Err(failure) = git.remove_worktree(&bare_dir, &target) {
            state.checks.push(CheckStatus {
                name: "cleanup worktree".into(),
                ok: false,
                detail: failure.message,
            });
        }
        if target.exists() {
            let _ = fs::remove_dir_all(&target);
        }
    }
    if let Err(failure) = git.delete_branch(&bare_dir, &state.branch) {
        state.checks.push(CheckStatus {
            name: "cleanup branch".into(),
            ok: false,
            detail: failure.message,
        });
    }
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

#[derive(Debug, Clone)]
struct AddExecutionState {
    repo_name: Option<String>,
    repo_root: PathBuf,
    branch: String,
    integration_branch: String,
    integration_worktree_path: Option<PathBuf>,
    worktree_path: Option<PathBuf>,
    checks: Vec<CheckStatus>,
    materialized: bool,
    created_worktree: bool,
}

impl AddExecutionState {
    fn new(request: &AddRequest) -> Self {
        Self {
            repo_name: None,
            repo_root: request.cwd.clone(),
            branch: request.branch_name.clone(),
            integration_branch: String::new(),
            integration_worktree_path: None,
            worktree_path: None,
            checks: Vec::new(),
            materialized: false,
            created_worktree: false,
        }
    }

    fn payload(&self) -> AddCommandPayload {
        AddCommandPayload {
            repo_name: self.repo_name.clone(),
            repo_root: Some(self.repo_root.clone()),
            branch: self.branch.clone(),
            worktree_path: self.worktree_path.clone(),
            materialized: self.materialized,
            checks: self.checks.clone(),
        }
    }

    fn fail(&self, failure: OperationFailure) -> Box<CommandFailure<AddCommandPayload>> {
        Box::new(CommandFailure {
            payload: self.payload(),
            failure,
        })
    }

    fn success(&self) -> AddCommandPayload {
        self.payload()
    }
}

use crate::commands::InitRequest;
use crate::config::{
    DEFAULT_INTEGRATION_BRANCH, DEFAULT_SKILLS_SYNC_SOURCE, RepoConfig, load_global_config,
    write_repo_config,
};
use crate::errors::{CommandFailure, FailureCategory, OperationFailure};
use crate::filesystem::{ensure_directory, ensure_file_contains_line};
use crate::git::{DiscoveredUntrackedPath, GitClient};
use crate::github::GitHubClient;
use crate::output::{CheckStatus, InitCommandPayload};
use crate::process::ToolNames;
use crate::state::{
    GraphNode, ManagedGraph, ManagedWorktreeRecord, UntrackedEntry, UntrackedEntryKind,
    UntrackedFilesManifest, WorktreeKind, WorktreesState, write_graph_state, write_manifest,
    write_worktrees_state,
};
use std::fs;
use std::path::{Path, PathBuf};

pub fn execute(
    request: InitRequest,
) -> Result<InitCommandPayload, Box<CommandFailure<InitCommandPayload>>> {
    let mut state = InitExecutionState::new(&request);
    let integration_branch = match resolve_integration_branch(&request) {
        Ok(branch) => branch,
        Err(failure) => return Err(state.fail(failure)),
    };
    state.integration_branch = integration_branch.clone();
    state.integration_worktree_path = Some(state.repo_root.join(&integration_branch));

    if let Err(failure) = validate_repo_name(&request.repo_name) {
        return Err(state.fail(failure));
    }
    if state.repo_root.exists() {
        return Err(state.fail(OperationFailure::new(
            FailureCategory::Filesystem,
            format!("target directory already exists: {}", state.repo_root.display()),
            "Init requires a fresh parent directory target. Remove the existing directory or choose a different parent directory.",
        )));
    }

    let tools = ToolNames::from_env();
    let git = GitClient::new(&tools);
    let github = GitHubClient::new(&tools);

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

    if let Err(failure) = github.ensure_available() {
        state.checks.push(CheckStatus {
            name: "gh available".into(),
            ok: false,
            detail: failure.message.clone(),
        });
        return Err(state.fail(failure));
    }
    state.checks.push(CheckStatus {
        name: "gh available".into(),
        ok: true,
        detail: "GitHub CLI is callable".into(),
    });

    if let Err(failure) = github.ensure_authenticated() {
        state.checks.push(CheckStatus {
            name: "gh auth status".into(),
            ok: false,
            detail: failure.message.clone(),
        });
        return Err(state.fail(failure));
    }
    state.checks.push(CheckStatus {
        name: "gh auth status".into(),
        ok: true,
        detail: "GitHub CLI is authenticated".into(),
    });

    let repo_view = match github.repo_view(&request.repo_name) {
        Ok(repo_view) => repo_view,
        Err(failure) => {
            state.checks.push(CheckStatus {
                name: "gh repo view".into(),
                ok: false,
                detail: failure.message.clone(),
            });
            return Err(state.fail(failure));
        }
    };
    state.checks.push(CheckStatus {
        name: "gh repo view".into(),
        ok: true,
        detail: format!("resolved {}", repo_view.name_with_owner),
    });

    if let Err(failure) = git.ensure_remote_branch_exists(&repo_view.clone_url, &integration_branch)
    {
        state.checks.push(CheckStatus {
            name: "integration branch".into(),
            ok: false,
            detail: failure.message.clone(),
        });
        return Err(state.fail(failure));
    }
    state.checks.push(CheckStatus {
        name: "integration branch".into(),
        ok: true,
        detail: format!("bootstrap will use `{integration_branch}`"),
    });

    if let Err(failure) = ensure_directory(&state.repo_root).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Filesystem,
            format!("failed to create repo root {}", state.repo_root.display()),
            format!("Ensure the parent directory is writable and retry. ({error})"),
        )
    }) {
        return Err(state.fail(failure));
    }
    state.created_repo_root = true;

    let result = run_bootstrap(&state, &git, &repo_view.clone_url);
    match result {
        Ok(untracked_paths) => {
            let manifest = build_manifest(&integration_branch, untracked_paths);
            if let Err(failure) = write_repo_state(&state, &manifest) {
                cleanup_partial_repo(&mut state, Some(&failure));
                return Err(state.fail(failure));
            }
            Ok(state.success())
        }
        Err(failure) => {
            cleanup_partial_repo(&mut state, Some(&failure));
            Err(state.fail(failure))
        }
    }
}

fn entry_kind_sort_rank(kind: &UntrackedEntryKind) -> u8 {
    match kind {
        UntrackedEntryKind::Directory => 0,
        UntrackedEntryKind::File => 1,
    }
}

fn is_same_or_descendant(path: &str, directory: &str) -> bool {
    path == directory
        || path
            .strip_prefix(directory)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn resolve_integration_branch(request: &InitRequest) -> Result<String, OperationFailure> {
    if let Some(branch) = request.integration_branch_override.clone() {
        return Ok(branch);
    }
    let config = load_global_config(&request.home_dir)?;
    Ok(config
        .integration_branch
        .unwrap_or_else(|| DEFAULT_INTEGRATION_BRANCH.to_string()))
}

fn validate_repo_name(repo_name: &str) -> Result<(), OperationFailure> {
    let looks_like_url = repo_name.contains("://")
        || repo_name.contains('/')
        || repo_name.contains(':')
        || repo_name.ends_with(".git");
    let allowed = repo_name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'));

    if repo_name.is_empty() || looks_like_url || !allowed {
        return Err(OperationFailure::new(
            FailureCategory::Config,
            format!("`{repo_name}` is not a valid bare repo name"),
            "Pass the bare repo name only, for example `zaz init zazz-skills`, not a clone URL or owner/repo path.",
        ));
    }

    Ok(())
}

fn run_bootstrap(
    state: &InitExecutionState,
    git: &GitClient,
    clone_url: &str,
) -> Result<Vec<DiscoveredUntrackedPath>, OperationFailure> {
    let bare_dir = state.repo_root.join(".bare");
    git.clone_bare(clone_url, &bare_dir)?;
    git.add_worktree_from_existing_branch(
        &bare_dir,
        state
            .integration_worktree_path
            .as_ref()
            .expect("integration worktree path should already be set"),
        &state.integration_branch,
    )?;

    let exclude_path = bare_dir.join("info").join("exclude");
    ensure_file_contains_line(&exclude_path, ".claude/skills/").map_err(|error| {
        OperationFailure::new(
            FailureCategory::Filesystem,
            format!(
                "failed to configure shared excludes at {}",
                exclude_path.display()
            ),
            format!("Ensure the bare repo is writable and retry. ({error})"),
        )
    })?;

    git.list_untracked(
        state
            .integration_worktree_path
            .as_ref()
            .expect("integration worktree path should already be set"),
    )
}

fn write_repo_state(
    state: &InitExecutionState,
    manifest: &UntrackedFilesManifest,
) -> Result<(), OperationFailure> {
    let repo_root = &state.repo_root;
    let zazz_dir = repo_root.join(".zazz");
    ensure_directory(&zazz_dir).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Filesystem,
            format!("failed to create {}", zazz_dir.display()),
            format!("Ensure the repo root is writable and retry. ({error})"),
        )
    })?;
    ensure_directory(&zazz_dir.join("conflicts")).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Filesystem,
            "failed to create .zazz/conflicts",
            format!("Ensure the repo root is writable and retry. ({error})"),
        )
    })?;
    ensure_directory(&zazz_dir.join("locks")).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Filesystem,
            "failed to create .zazz/locks",
            format!("Ensure the repo root is writable and retry. ({error})"),
        )
    })?;

    write_repo_config(
        repo_root,
        &RepoConfig {
            repo_name: state.repo_name.clone(),
            integration_branch: state.integration_branch.clone(),
            integration_worktree: state.integration_branch.clone(),
            skills_sync_source: DEFAULT_SKILLS_SYNC_SOURCE.to_string(),
        },
    )?;
    write_graph_state(
        repo_root,
        &ManagedGraph {
            version: 1,
            nodes: vec![GraphNode {
                branch: state.integration_branch.clone(),
                worktree: state.integration_branch.clone(),
                path: display_path(
                    state
                        .integration_worktree_path
                        .as_ref()
                        .expect("integration worktree path should already be set"),
                ),
                parent_branch: None,
                kind: WorktreeKind::Integration,
            }],
        },
    )?;
    write_worktrees_state(
        repo_root,
        &WorktreesState {
            version: 1,
            integration_branch: state.integration_branch.clone(),
            items: vec![ManagedWorktreeRecord {
                branch: state.integration_branch.clone(),
                worktree: state.integration_branch.clone(),
                path: display_path(
                    state
                        .integration_worktree_path
                        .as_ref()
                        .expect("integration worktree path should already be set"),
                ),
                kind: WorktreeKind::Integration,
            }],
        },
    )?;
    write_manifest(repo_root, manifest)?;
    Ok(())
}

pub fn build_manifest(
    source_worktree: &str,
    discovered_paths: Vec<DiscoveredUntrackedPath>,
) -> UntrackedFilesManifest {
    let mut entries = discovered_paths
        .into_iter()
        .map(|entry| UntrackedEntry {
            path: entry.path,
            kind: entry.kind,
            required: false,
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| entry_kind_sort_rank(&left.kind).cmp(&entry_kind_sort_rank(&right.kind)))
    });
    entries.dedup_by(|left, right| left.path == right.path && left.kind == right.kind);

    let mut filtered = Vec::with_capacity(entries.len());
    let mut recorded_directories: Vec<String> = Vec::new();
    for entry in entries {
        if recorded_directories
            .iter()
            .any(|directory| is_same_or_descendant(&entry.path, directory))
        {
            continue;
        }

        if entry.kind == UntrackedEntryKind::Directory {
            recorded_directories.push(entry.path.clone());
        }
        filtered.push(entry);
    }

    UntrackedFilesManifest {
        version: 1,
        source_worktree: source_worktree.to_string(),
        entries: filtered,
    }
}

fn cleanup_partial_repo(state: &mut InitExecutionState, failure: Option<&OperationFailure>) {
    if !state.created_repo_root {
        return;
    }
    if let Err(error) = fs::remove_dir_all(&state.repo_root) {
        state.checks.push(CheckStatus {
            name: "cleanup".into(),
            ok: false,
            detail: format!(
                "failed to remove partial repo root {} after error {}: {}",
                state.repo_root.display(),
                failure
                    .map(|item| item.message.as_str())
                    .unwrap_or("unknown failure"),
                error
            ),
        });
    }
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

#[derive(Debug, Clone)]
struct InitExecutionState {
    repo_name: String,
    repo_root: PathBuf,
    integration_branch: String,
    integration_worktree_path: Option<PathBuf>,
    checks: Vec<CheckStatus>,
    created_repo_root: bool,
}

impl InitExecutionState {
    fn new(request: &InitRequest) -> Self {
        Self {
            repo_name: request.repo_name.clone(),
            repo_root: request.cwd.join(&request.repo_name),
            integration_branch: DEFAULT_INTEGRATION_BRANCH.to_string(),
            integration_worktree_path: None,
            checks: Vec::new(),
            created_repo_root: false,
        }
    }

    fn payload(&self, initialized: bool) -> InitCommandPayload {
        InitCommandPayload {
            repo_name: self.repo_name.clone(),
            repo_root: Some(self.repo_root.clone()),
            integration_branch: self.integration_branch.clone(),
            integration_worktree_path: self.integration_worktree_path.clone(),
            initialized,
            checks: self.checks.clone(),
        }
    }

    fn fail(&self, failure: OperationFailure) -> Box<CommandFailure<InitCommandPayload>> {
        Box::new(CommandFailure {
            payload: self.payload(false),
            failure,
        })
    }

    fn success(&self) -> InitCommandPayload {
        self.payload(true)
    }
}

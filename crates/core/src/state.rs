use crate::errors::{FailureCategory, OperationFailure};
use crate::filesystem::write_text_atomic;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedGraph {
    pub version: u8,
    pub nodes: Vec<GraphNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub branch: String,
    pub worktree: String,
    pub path: String,
    pub parent_branch: Option<String>,
    pub kind: WorktreeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorktreeKind {
    Integration,
    Feature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorktreesState {
    pub version: u8,
    pub integration_branch: String,
    pub items: Vec<ManagedWorktreeRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedWorktreeRecord {
    pub branch: String,
    pub worktree: String,
    pub path: String,
    pub kind: WorktreeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UntrackedFilesManifest {
    pub version: u8,
    pub source_worktree: String,
    pub entries: Vec<UntrackedEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct UntrackedEntry {
    pub path: String,
    pub kind: UntrackedEntryKind,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum UntrackedEntryKind {
    File,
    Directory,
}

pub fn graph_state_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".zazz").join("graph.json")
}

pub fn worktrees_state_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".zazz").join("worktrees.json")
}

pub fn manifest_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".zazz").join("untracked-files.json")
}

pub fn write_graph_state(repo_root: &Path, graph: &ManagedGraph) -> Result<(), OperationFailure> {
    write_json_state(&graph_state_path(repo_root), graph)
}

pub fn write_worktrees_state(
    repo_root: &Path,
    worktrees: &WorktreesState,
) -> Result<(), OperationFailure> {
    write_json_state(&worktrees_state_path(repo_root), worktrees)
}

pub fn write_manifest(
    repo_root: &Path,
    manifest: &UntrackedFilesManifest,
) -> Result<(), OperationFailure> {
    write_json_state(&manifest_path(repo_root), manifest)
}

pub fn read_graph_state(repo_root: &Path) -> Result<ManagedGraph, OperationFailure> {
    read_json_state(&graph_state_path(repo_root))
}

pub fn read_worktrees_state(repo_root: &Path) -> Result<WorktreesState, OperationFailure> {
    read_json_state(&worktrees_state_path(repo_root))
}

pub fn read_manifest(repo_root: &Path) -> Result<UntrackedFilesManifest, OperationFailure> {
    read_json_state(&manifest_path(repo_root))
}

fn write_json_state<T: Serialize>(path: &Path, value: &T) -> Result<(), OperationFailure> {
    let contents = serde_json::to_string_pretty(value).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Filesystem,
            format!("failed to serialize {}", path.display()),
            format!("Review the generated repo-local state schema. ({error})"),
        )
    })?;

    write_text_atomic(path, &contents).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Filesystem,
            format!("failed to write {}", path.display()),
            format!("Ensure the repo root is writable and retry. ({error})"),
        )
    })
}

fn read_json_state<T>(path: &Path) -> Result<T, OperationFailure>
where
    T: for<'de> Deserialize<'de>,
{
    let contents = fs::read_to_string(path).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Config,
            format!("failed to read {}", path.display()),
            format!("Run `zaz init` successfully before using this repo-local state. ({error})"),
        )
    })?;

    serde_json::from_str(&contents).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Config,
            format!("failed to parse {}", path.display()),
            format!("Fix the repo-local state file or rerun `zaz init`. ({error})"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{UntrackedEntry, UntrackedEntryKind, UntrackedFilesManifest};

    #[test]
    fn untracked_manifest_serializes_expected_schema() {
        let manifest = UntrackedFilesManifest {
            version: 1,
            source_worktree: "dev".into(),
            entries: vec![UntrackedEntry {
                path: ".env".into(),
                kind: UntrackedEntryKind::File,
                required: false,
            }],
        };

        let json = serde_json::to_string_pretty(&manifest).expect("manifest should serialize");

        assert!(json.contains("\"version\": 1"));
        assert!(json.contains("\"sourceWorktree\": \"dev\""));
        assert!(json.contains("\"path\": \".env\""));
        assert!(json.contains("\"kind\": \"file\""));
        assert!(json.contains("\"required\": false"));
    }
}

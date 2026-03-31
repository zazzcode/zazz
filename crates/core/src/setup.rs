use crate::config::RepoConfig;
use crate::errors::{FailureCategory, OperationFailure};
use crate::filesystem::{copy_directory_recursive, ensure_directory, ensure_file_contains_line};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkillsSyncReport {
    pub copied_skills: Vec<String>,
    pub skipped_missing_source: bool,
}

pub fn ensure_shared_excludes(repo_root: &Path) -> Result<(), OperationFailure> {
    let exclude_path = repo_root.join(".bare").join("info").join("exclude");
    ensure_file_contains_line(&exclude_path, ".claude/skills/").map_err(|error| {
        OperationFailure::new(
            FailureCategory::Filesystem,
            format!(
                "failed to configure shared excludes at {}",
                exclude_path.display()
            ),
            format!("Ensure the bare repo is writable and retry. ({error})"),
        )
    })
}

pub fn sync_local_skills(
    repo_root: &Path,
    config: &RepoConfig,
    target_worktree: &Path,
) -> Result<SkillsSyncReport, OperationFailure> {
    let source_root = resolve_skills_source(repo_root, &config.skills_sync_source);
    if !source_root.exists() {
        return Ok(SkillsSyncReport {
            copied_skills: Vec::new(),
            skipped_missing_source: true,
        });
    }

    let target_root = target_worktree.join(".claude").join("skills");
    ensure_directory(&target_root).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Filesystem,
            format!("failed to create {}", target_root.display()),
            format!("Ensure the new worktree is writable and retry. ({error})"),
        )
    })?;

    let mut copied_skills = Vec::new();
    for entry in fs::read_dir(&source_root).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Filesystem,
            format!("failed to read skills source {}", source_root.display()),
            format!("Ensure the configured skills source is readable. ({error})"),
        )
    })? {
        let entry = entry.map_err(|error| {
            OperationFailure::new(
                FailureCategory::Filesystem,
                format!("failed to inspect skills under {}", source_root.display()),
                format!("Ensure the configured skills source is readable. ({error})"),
            )
        })?;
        if !entry
            .file_type()
            .map_err(|error| {
                OperationFailure::new(
                    FailureCategory::Filesystem,
                    format!("failed to inspect {}", entry.path().display()),
                    format!("Ensure the configured skills source is readable. ({error})"),
                )
            })?
            .is_dir()
        {
            continue;
        }

        let name = entry.file_name().to_string_lossy().into_owned();
        let destination = target_root.join(&name);
        copy_directory_recursive(&entry.path(), &destination).map_err(|error| {
            OperationFailure::new(
                FailureCategory::Filesystem,
                format!("failed to sync skill {}", name),
                format!("Ensure the new worktree is writable and retry. ({error})"),
            )
        })?;
        copied_skills.push(name);
    }
    copied_skills.sort();

    Ok(SkillsSyncReport {
        copied_skills,
        skipped_missing_source: false,
    })
}

fn resolve_skills_source(repo_root: &Path, configured_source: &str) -> PathBuf {
    repo_root.join(configured_source)
}

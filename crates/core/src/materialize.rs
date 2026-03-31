use crate::errors::{FailureCategory, OperationFailure};
use crate::filesystem::{copy_directory_recursive, copy_file};
use crate::state::{UntrackedEntryKind, UntrackedFilesManifest};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MaterializationReport {
    pub copied: Vec<String>,
    pub missing_optional: Vec<String>,
}

pub fn materialize(
    source_root: &Path,
    target_root: &Path,
    manifest: &UntrackedFilesManifest,
) -> Result<MaterializationReport, OperationFailure> {
    let mut report = MaterializationReport::default();

    for entry in &manifest.entries {
        let source_path = source_root.join(&entry.path);
        let target_path = target_root.join(&entry.path);

        if !source_path.exists() {
            if entry.required {
                return Err(OperationFailure::new(
                    FailureCategory::Materialization,
                    format!(
                        "required manifest path is missing from the integration worktree: {}",
                        entry.path
                    ),
                    "Restore the required source path in the integration worktree or update .zazz/untracked-files.json.",
                ));
            }
            report.missing_optional.push(entry.path.clone());
            continue;
        }

        match entry.kind {
            UntrackedEntryKind::File => {
                if !source_path.is_file() {
                    return Err(OperationFailure::new(
                        FailureCategory::Materialization,
                        format!(
                            "manifest entry {} is marked as a file but is not a file",
                            entry.path
                        ),
                        "Fix .zazz/untracked-files.json so the entry kind matches the integration worktree.",
                    ));
                }
                copy_file(&source_path, &target_path).map_err(|error| {
                    OperationFailure::new(
                        FailureCategory::Materialization,
                        format!("failed to copy {}", entry.path),
                        format!("Ensure the target worktree is writable and retry. ({error})"),
                    )
                })?;
                report.copied.push(entry.path.clone());
            }
            UntrackedEntryKind::Directory => {
                if !source_path.is_dir() {
                    return Err(OperationFailure::new(
                        FailureCategory::Materialization,
                        format!(
                            "manifest entry {} is marked as a directory but is not a directory",
                            entry.path
                        ),
                        "Fix .zazz/untracked-files.json so the entry kind matches the integration worktree.",
                    ));
                }
                copy_directory_recursive(&source_path, &target_path).map_err(|error| {
                    OperationFailure::new(
                        FailureCategory::Materialization,
                        format!("failed to copy directory {}", entry.path),
                        format!("Ensure the target worktree is writable and retry. ({error})"),
                    )
                })?;
                report.copied.push(entry.path.clone());
            }
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::materialize;
    use crate::state::{UntrackedEntry, UntrackedEntryKind, UntrackedFilesManifest};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn copies_only_manifest_listed_entries() {
        let temp_dir = TempDir::new().expect("temp dir");
        let source = temp_dir.path().join("source");
        let target = temp_dir.path().join("target");
        fs::create_dir_all(source.join("dir/nested")).expect("create source dir");
        fs::write(source.join(".env"), "A=1\n").expect("write file");
        fs::write(source.join("dir/nested/file.txt"), "nested\n").expect("write nested");
        fs::write(source.join("skip.txt"), "skip\n").expect("write skip");

        let report = materialize(
            &source,
            &target,
            &UntrackedFilesManifest {
                version: 1,
                source_worktree: "main".into(),
                entries: vec![
                    UntrackedEntry {
                        path: ".env".into(),
                        kind: UntrackedEntryKind::File,
                        required: false,
                    },
                    UntrackedEntry {
                        path: "dir".into(),
                        kind: UntrackedEntryKind::Directory,
                        required: false,
                    },
                ],
            },
        )
        .expect("materialization should succeed");

        assert_eq!(report.copied, vec![".env", "dir"]);
        assert!(target.join(".env").exists());
        assert!(target.join("dir/nested/file.txt").exists());
        assert!(!target.join("skip.txt").exists());
    }
}

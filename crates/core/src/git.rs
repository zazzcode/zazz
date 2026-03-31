use crate::errors::{FailureCategory, OperationFailure};
use crate::process::{CommandSpec, ToolNames, path_to_os_string, run};
use crate::state::UntrackedEntryKind;
use std::ffi::OsString;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GitClient {
    program: OsString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredUntrackedPath {
    pub path: String,
    pub kind: UntrackedEntryKind,
}

impl GitClient {
    pub fn new(tools: &ToolNames) -> Self {
        Self {
            program: tools.git.clone(),
        }
    }

    pub fn ensure_available(&self) -> Result<(), OperationFailure> {
        let spec = CommandSpec {
            program: self.program.clone(),
            args: vec![OsString::from("--version")],
            cwd: None,
        };
        let output = run(&spec).map_err(|error| {
            OperationFailure::new(
                FailureCategory::Tooling,
                "git is not installed or not callable",
                format!("Install git so `{}` can run. ({error})", spec.display()),
            )
        })?;
        if output.status.success() {
            Ok(())
        } else {
            Err(OperationFailure::new(
                FailureCategory::Tooling,
                "git is not ready",
                format!(
                    "Ensure git is installed and callable. stderr: {}",
                    trim_output(&output.stderr)
                ),
            ))
        }
    }

    pub fn ensure_remote_branch_exists(
        &self,
        remote: &str,
        branch: &str,
    ) -> Result<(), OperationFailure> {
        let spec = CommandSpec {
            program: self.program.clone(),
            args: vec![
                OsString::from("ls-remote"),
                OsString::from("--exit-code"),
                OsString::from("--heads"),
                OsString::from(remote),
                OsString::from(branch),
            ],
            cwd: None,
        };
        let output = run(&spec).map_err(|error| {
            OperationFailure::new(
                FailureCategory::Git,
                format!("failed to check remote branch `{branch}`"),
                format!("Verify git can reach the resolved repo and retry. ({error})"),
            )
        })?;
        if output.status.success() {
            Ok(())
        } else {
            Err(OperationFailure::new(
                FailureCategory::Config,
                format!("integration branch `{branch}` is not available for bootstrap"),
                format!(
                    "Pass --integration with a valid branch or set top-level integration_branch in ~/.zazz/config.toml. stderr: {}",
                    trim_output(&output.stderr)
                ),
            ))
        }
    }

    pub fn clone_bare(&self, remote: &str, bare_dir: &Path) -> Result<(), OperationFailure> {
        let spec = CommandSpec {
            program: self.program.clone(),
            args: vec![
                OsString::from("clone"),
                OsString::from("--bare"),
                OsString::from(remote),
                path_to_os_string(bare_dir),
            ],
            cwd: None,
        };
        let output = run(&spec).map_err(|error| {
            OperationFailure::new(
                FailureCategory::Git,
                "failed to start bare clone",
                format!("Verify git can clone the resolved repo. ({error})"),
            )
        })?;
        if output.status.success() {
            Ok(())
        } else {
            Err(OperationFailure::new(
                FailureCategory::Git,
                "bare clone failed",
                format!(
                    "Verify git access to the resolved repo. stderr: {}",
                    trim_output(&output.stderr)
                ),
            ))
        }
    }

    pub fn add_worktree_from_existing_branch(
        &self,
        bare_dir: &Path,
        target_path: &Path,
        branch: &str,
    ) -> Result<(), OperationFailure> {
        let spec = CommandSpec {
            program: self.program.clone(),
            args: vec![
                OsString::from(format!("--git-dir={}", bare_dir.display())),
                OsString::from("worktree"),
                OsString::from("add"),
                path_to_os_string(target_path),
                OsString::from(branch),
            ],
            cwd: None,
        };
        let output = run(&spec).map_err(|error| {
            OperationFailure::new(
                FailureCategory::Git,
                "failed to create integration worktree",
                format!("Verify git can create a linked worktree. ({error})"),
            )
        })?;
        if output.status.success() {
            Ok(())
        } else {
            Err(OperationFailure::new(
                FailureCategory::Git,
                "git worktree add failed during init",
                format!(
                    "Verify the resolved integration branch is available locally. stderr: {}",
                    trim_output(&output.stderr)
                ),
            ))
        }
    }

    pub fn list_untracked(
        &self,
        worktree_path: &Path,
    ) -> Result<Vec<DiscoveredUntrackedPath>, OperationFailure> {
        let spec = CommandSpec {
            program: self.program.clone(),
            args: vec![
                OsString::from("status"),
                OsString::from("--porcelain"),
                OsString::from("--untracked-files=normal"),
            ],
            cwd: Some(worktree_path.to_path_buf()),
        };
        let output = run(&spec).map_err(|error| {
            OperationFailure::new(
                FailureCategory::Git,
                "failed to inspect untracked files in the integration worktree",
                format!("Verify git can inspect the integration worktree. ({error})"),
            )
        })?;
        if !output.status.success() {
            return Err(OperationFailure::new(
                FailureCategory::Git,
                "git status failed while seeding the untracked-files manifest",
                format!(
                    "Verify the integration worktree is healthy. stderr: {}",
                    trim_output(&output.stderr)
                ),
            ));
        }

        let mut entries = output
            .stdout
            .lines()
            .filter_map(parse_untracked_status_line)
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(entries)
    }

    pub fn fetch_prune_origin(&self, bare_dir: &Path) -> Result<(), OperationFailure> {
        let output = self.run_git_dir(
            bare_dir,
            [
                OsString::from("fetch"),
                OsString::from("--prune"),
                OsString::from("origin"),
            ],
            FailureCategory::Git,
            "failed to fetch origin",
            "Verify network and remote access, then retry `zaz add`.",
        )?;
        ensure_success(
            output.status.success(),
            FailureCategory::Git,
            "failed to fetch origin",
            format!(
                "Verify network and remote access. stderr: {}",
                trim_output(&output.stderr)
            ),
        )
    }

    pub fn pull_ff_only(&self, worktree_path: &Path) -> Result<(), OperationFailure> {
        let spec = CommandSpec {
            program: self.program.clone(),
            args: vec![OsString::from("pull"), OsString::from("--ff-only")],
            cwd: Some(worktree_path.to_path_buf()),
        };
        let output = run(&spec).map_err(|error| {
            OperationFailure::new(
                FailureCategory::Git,
                "failed to start `git pull --ff-only`",
                format!("Verify the integration worktree is available and retry. ({error})"),
            )
        })?;
        ensure_success(
            output.status.success(),
            FailureCategory::Git,
            "failed to fast-forward the local integration worktree",
            format!(
                "Resolve the local integration worktree state and retry. stderr: {}",
                trim_output(&output.stderr)
            ),
        )
    }

    pub fn local_branch_exists(
        &self,
        bare_dir: &Path,
        branch: &str,
    ) -> Result<bool, OperationFailure> {
        let output = self.run_git_dir(
            bare_dir,
            [
                OsString::from("show-ref"),
                OsString::from("--verify"),
                OsString::from("--quiet"),
                OsString::from(format!("refs/heads/{branch}")),
            ],
            FailureCategory::Git,
            "failed to inspect local branches",
            "Verify the bare repo is healthy and retry.",
        )?;
        Ok(output.status.success())
    }

    pub fn add_worktree_new_branch(
        &self,
        bare_dir: &Path,
        target_path: &Path,
        branch: &str,
        start_point: &str,
    ) -> Result<(), OperationFailure> {
        let output = self.run_git_dir(
            bare_dir,
            [
                OsString::from("worktree"),
                OsString::from("add"),
                path_to_os_string(target_path),
                OsString::from("-b"),
                OsString::from(branch),
                OsString::from(start_point),
            ],
            FailureCategory::Git,
            "failed to create the new worktree",
            "Verify the requested branch name and local git state, then retry `zaz add`.",
        )?;
        ensure_success(
            output.status.success(),
            FailureCategory::Git,
            "git worktree add failed",
            format!(
                "Verify the requested branch name and local git state. stderr: {}",
                trim_output(&output.stderr)
            ),
        )
    }

    pub fn remove_worktree(
        &self,
        bare_dir: &Path,
        target_path: &Path,
    ) -> Result<(), OperationFailure> {
        let output = self.run_git_dir(
            bare_dir,
            [
                OsString::from("worktree"),
                OsString::from("remove"),
                OsString::from("--force"),
                path_to_os_string(target_path),
            ],
            FailureCategory::Git,
            "failed to remove a partially created worktree",
            "Remove the partial worktree manually if cleanup keeps failing.",
        )?;
        ensure_success(
            output.status.success(),
            FailureCategory::Git,
            "git worktree remove failed during cleanup",
            format!(
                "Remove the partial worktree manually if needed. stderr: {}",
                trim_output(&output.stderr)
            ),
        )
    }

    pub fn delete_branch(&self, bare_dir: &Path, branch: &str) -> Result<(), OperationFailure> {
        let output = self.run_git_dir(
            bare_dir,
            [
                OsString::from("branch"),
                OsString::from("-D"),
                OsString::from(branch),
            ],
            FailureCategory::Git,
            "failed to delete a partially created branch",
            "Delete the partial branch manually if cleanup keeps failing.",
        )?;
        ensure_success(
            output.status.success(),
            FailureCategory::Git,
            "git branch -D failed during cleanup",
            format!(
                "Delete the partial branch manually if needed. stderr: {}",
                trim_output(&output.stderr)
            ),
        )
    }

    pub fn current_branch(&self, worktree_path: &Path) -> Result<String, OperationFailure> {
        let spec = CommandSpec {
            program: self.program.clone(),
            args: vec![
                OsString::from("rev-parse"),
                OsString::from("--abbrev-ref"),
                OsString::from("HEAD"),
            ],
            cwd: Some(worktree_path.to_path_buf()),
        };
        let output = run(&spec).map_err(|error| {
            OperationFailure::new(
                FailureCategory::Git,
                "failed to determine the checked-out branch",
                format!("Verify the new worktree is accessible and retry. ({error})"),
            )
        })?;
        if output.status.success() {
            Ok(output.stdout.trim().to_string())
        } else {
            Err(OperationFailure::new(
                FailureCategory::Git,
                "failed to determine the checked-out branch",
                format!(
                    "Verify the new worktree is accessible. stderr: {}",
                    trim_output(&output.stderr)
                ),
            ))
        }
    }

    fn run_git_dir<I>(
        &self,
        bare_dir: &Path,
        args: I,
        category: FailureCategory,
        message: &str,
        remediation: &str,
    ) -> Result<crate::process::CommandOutput, OperationFailure>
    where
        I: IntoIterator<Item = OsString>,
    {
        let spec = CommandSpec {
            program: self.program.clone(),
            args: std::iter::once(OsString::from(format!("--git-dir={}", bare_dir.display())))
                .chain(args)
                .collect(),
            cwd: None,
        };
        run(&spec).map_err(|error| {
            OperationFailure::new(category, message, format!("{remediation} ({error})"))
        })
    }
}

fn parse_untracked_status_line(line: &str) -> Option<DiscoveredUntrackedPath> {
    let path = line.strip_prefix("?? ")?;
    if path.is_empty() {
        return None;
    }
    if let Some(directory) = path.strip_suffix('/') {
        return Some(DiscoveredUntrackedPath {
            path: directory.to_string(),
            kind: UntrackedEntryKind::Directory,
        });
    }
    Some(DiscoveredUntrackedPath {
        path: path.to_string(),
        kind: UntrackedEntryKind::File,
    })
}

fn trim_output(output: &str) -> String {
    output.trim().to_string()
}

fn ensure_success(
    success: bool,
    category: FailureCategory,
    message: &str,
    remediation: String,
) -> Result<(), OperationFailure> {
    if success {
        Ok(())
    } else {
        Err(OperationFailure::new(category, message, remediation))
    }
}

#[cfg(test)]
mod tests {
    use super::{DiscoveredUntrackedPath, parse_untracked_status_line};
    use crate::state::UntrackedEntryKind;

    #[test]
    fn parses_file_and_directory_untracked_status_lines() {
        assert_eq!(
            parse_untracked_status_line("?? .env"),
            Some(DiscoveredUntrackedPath {
                path: ".env".into(),
                kind: UntrackedEntryKind::File,
            })
        );
        assert_eq!(
            parse_untracked_status_line("?? local-dir/"),
            Some(DiscoveredUntrackedPath {
                path: "local-dir".into(),
                kind: UntrackedEntryKind::Directory,
            })
        );
        assert_eq!(parse_untracked_status_line(" M tracked.txt"), None);
    }
}

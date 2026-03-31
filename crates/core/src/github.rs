use crate::errors::{FailureCategory, OperationFailure};
use crate::process::{CommandSpec, ToolNames, run};
use serde::Deserialize;
use std::ffi::OsString;

#[derive(Debug, Clone)]
pub struct GitHubClient {
    program: OsString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoView {
    pub name_with_owner: String,
    pub clone_url: String,
}

impl GitHubClient {
    pub fn new(tools: &ToolNames) -> Self {
        Self {
            program: tools.gh.clone(),
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
                "GitHub CLI is not installed or not callable",
                format!(
                    "Install GitHub CLI so `{}` can run. ({error})",
                    spec.display()
                ),
            )
        })?;
        if output.status.success() {
            Ok(())
        } else {
            Err(OperationFailure::new(
                FailureCategory::Tooling,
                "GitHub CLI is not ready",
                format!(
                    "Ensure GitHub CLI is installed and callable. stderr: {}",
                    trim_output(&output.stderr)
                ),
            ))
        }
    }

    pub fn ensure_authenticated(&self) -> Result<(), OperationFailure> {
        let spec = CommandSpec {
            program: self.program.clone(),
            args: vec![OsString::from("auth"), OsString::from("status")],
            cwd: None,
        };
        let output = run(&spec).map_err(|error| {
            OperationFailure::new(
                FailureCategory::Tooling,
                "GitHub CLI could not run `gh auth status`",
                format!("Install and configure GitHub CLI. ({error})"),
            )
        })?;
        if output.status.success() {
            Ok(())
        } else {
            Err(OperationFailure::new(
                FailureCategory::Auth,
                "GitHub CLI is not authenticated",
                format!(
                    "Authenticate GitHub CLI and verify the expected PAT or host login is configured. stderr: {}",
                    trim_output(&output.stderr)
                ),
            ))
        }
    }

    pub fn repo_view(&self, repo_name: &str) -> Result<RepoView, OperationFailure> {
        let spec = CommandSpec {
            program: self.program.clone(),
            args: vec![
                OsString::from("repo"),
                OsString::from("view"),
                OsString::from(repo_name),
                OsString::from("--json"),
                OsString::from("nameWithOwner,url"),
            ],
            cwd: None,
        };
        let output = run(&spec).map_err(|error| {
            OperationFailure::new(
                FailureCategory::Tooling,
                "GitHub CLI could not run `gh repo view`",
                format!("Install and configure GitHub CLI. ({error})"),
            )
        })?;
        if !output.status.success() {
            return Err(OperationFailure::new(
                FailureCategory::RepoResolution,
                format!("`gh repo view {repo_name}` failed"),
                format!(
                    "Ensure the named repo resolves through the authenticated GitHub CLI context. stderr: {}",
                    trim_output(&output.stderr)
                ),
            ));
        }

        let parsed: RepoViewResponse = serde_json::from_str(&output.stdout).map_err(|error| {
            OperationFailure::new(
                FailureCategory::RepoResolution,
                "failed to parse `gh repo view` output",
                format!("Retry once GitHub CLI returns valid JSON for the named repo. ({error})"),
            )
        })?;
        Ok(RepoView {
            name_with_owner: parsed.name_with_owner,
            clone_url: normalize_clone_url(parsed.url),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepoViewResponse {
    name_with_owner: String,
    url: String,
}

fn normalize_clone_url(raw: String) -> String {
    if raw.starts_with("http://") || raw.starts_with("https://") {
        if raw.ends_with(".git") {
            raw
        } else {
            format!("{raw}.git")
        }
    } else {
        raw
    }
}

fn trim_output(output: &str) -> String {
    output.trim().to_string()
}

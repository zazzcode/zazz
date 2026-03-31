use crate::errors::{FailureCategory, ZazzlesError};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandRenderMode {
    Human,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedCommandOutput {
    pub success: bool,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckStatus {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailureInfo {
    pub category: FailureCategory,
    pub message: String,
    pub remediation: String,
}

pub trait HumanSummary {
    fn success_summary(&self) -> String;
    fn failure_summary(&self, failure: &FailureInfo) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitCommandPayload {
    pub repo_name: String,
    pub repo_root: Option<PathBuf>,
    pub integration_branch: String,
    pub integration_worktree_path: Option<PathBuf>,
    pub initialized: bool,
    pub checks: Vec<CheckStatus>,
}

impl HumanSummary for InitCommandPayload {
    fn success_summary(&self) -> String {
        format!(
            "Initialized {} at {} using integration branch {}.\n{}",
            self.repo_name,
            display_path(&self.repo_root),
            self.integration_branch,
            render_checks(&self.checks)
        )
    }

    fn failure_summary(&self, failure: &FailureInfo) -> String {
        format!(
            "Init failed for {}.\nCategory: {}\nReason: {}\nNext step: {}\n{}",
            self.repo_name,
            failure.category.as_str(),
            failure.message,
            failure.remediation,
            render_checks(&self.checks)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCommandPayload {
    pub repo_name: Option<String>,
    pub repo_root: Option<PathBuf>,
    pub branch: String,
    pub worktree_path: Option<PathBuf>,
    pub materialized: bool,
    pub checks: Vec<CheckStatus>,
}

impl HumanSummary for AddCommandPayload {
    fn success_summary(&self) -> String {
        format!(
            "Created worktree {} at {}.\n{}",
            self.branch,
            display_path(&self.worktree_path),
            render_checks(&self.checks)
        )
    }

    fn failure_summary(&self, failure: &FailureInfo) -> String {
        format!(
            "Add failed for branch {}.\nCategory: {}\nReason: {}\nNext step: {}\n{}",
            self.branch,
            failure.category.as_str(),
            failure.message,
            failure.remediation,
            render_checks(&self.checks)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandEnvelope<T>
where
    T: Serialize,
{
    pub version: u8,
    pub command: String,
    pub success: bool,
    pub failure: Option<FailureInfo>,
    #[serde(flatten)]
    pub payload: T,
}

impl<T> CommandEnvelope<T>
where
    T: Serialize + HumanSummary,
{
    pub fn success(command: &str, payload: T) -> Self {
        Self {
            version: 1,
            command: command.to_string(),
            success: true,
            failure: None,
            payload,
        }
    }

    pub fn failure(
        command: &str,
        payload: T,
        category: FailureCategory,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Self {
        Self {
            version: 1,
            command: command.to_string(),
            success: false,
            failure: Some(FailureInfo {
                category,
                message: message.into(),
                remediation: remediation.into(),
            }),
            payload,
        }
    }

    pub fn render(&self, mode: CommandRenderMode) -> Result<RenderedCommandOutput, ZazzlesError> {
        let body = match mode {
            CommandRenderMode::Human => self.render_human(),
            CommandRenderMode::Json => serde_json::to_string_pretty(self)
                .map_err(|source| ZazzlesError::JsonSerialization { source })?,
        };

        Ok(RenderedCommandOutput {
            success: self.success,
            body,
        })
    }

    fn render_human(&self) -> String {
        match &self.failure {
            Some(failure) => self.payload.failure_summary(failure),
            None => self.payload.success_summary(),
        }
    }
}

fn display_path(path: &Option<PathBuf>) -> String {
    path.as_ref()
        .map(|value| value.display().to_string())
        .unwrap_or_else(|| "(unknown path)".to_string())
}

fn render_checks(checks: &[CheckStatus]) -> String {
    if checks.is_empty() {
        return "Checks: none recorded".to_string();
    }

    let mut lines = vec!["Checks:".to_string()];
    for check in checks {
        let status = if check.ok { "ok" } else { "failed" };
        lines.push(format!("- {}: {} ({})", check.name, status, check.detail));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{
        AddCommandPayload, CheckStatus, CommandEnvelope, CommandRenderMode, InitCommandPayload,
    };
    use crate::errors::FailureCategory;
    use serde_json::Value;
    use std::path::PathBuf;

    #[test]
    fn init_json_envelope_contains_required_fields() {
        let envelope = CommandEnvelope::success(
            "init",
            InitCommandPayload {
                repo_name: "zazzles".into(),
                repo_root: Some(PathBuf::from("/tmp/zazzles")),
                integration_branch: "main".into(),
                integration_worktree_path: Some(PathBuf::from("/tmp/zazzles/main")),
                initialized: true,
                checks: vec![CheckStatus {
                    name: "gh auth status".into(),
                    ok: true,
                    detail: "authenticated".into(),
                }],
            },
        );

        let rendered = envelope
            .render(CommandRenderMode::Json)
            .expect("json rendering should succeed");
        let value: Value =
            serde_json::from_str(&rendered.body).expect("rendered output should be valid json");

        assert_eq!(value["version"], 1);
        assert_eq!(value["command"], "init");
        assert_eq!(value["success"], true);
        assert_eq!(value["failure"], Value::Null);
        assert_eq!(value["repoName"], "zazzles");
        assert_eq!(value["repoRoot"], "/tmp/zazzles");
        assert_eq!(value["integrationBranch"], "main");
        assert_eq!(value["integrationWorktreePath"], "/tmp/zazzles/main");
        assert_eq!(value["initialized"], true);
        assert!(value["checks"].is_array());
    }

    #[test]
    fn add_failure_json_envelope_contains_category_and_remediation() {
        let envelope = CommandEnvelope::failure(
            "add",
            AddCommandPayload {
                repo_name: Some("zazzles".into()),
                repo_root: Some(PathBuf::from("/tmp/zazzles")),
                branch: "feature-a".into(),
                worktree_path: None,
                materialized: false,
                checks: Vec::new(),
            },
            FailureCategory::Git,
            "pull --ff-only failed",
            "Resolve local integration worktree divergence and retry `zaz add`.",
        );

        let rendered = envelope
            .render(CommandRenderMode::Json)
            .expect("json rendering should succeed");
        let value: Value =
            serde_json::from_str(&rendered.body).expect("rendered output should be valid json");

        assert_eq!(value["command"], "add");
        assert_eq!(value["success"], false);
        assert_eq!(value["failure"]["category"], "git");
        assert_eq!(value["failure"]["message"], "pull --ff-only failed");
        assert_eq!(
            value["failure"]["remediation"],
            "Resolve local integration worktree divergence and retry `zaz add`."
        );
        assert_eq!(value["branch"], "feature-a");
        assert_eq!(value["materialized"], false);
    }

    #[test]
    fn human_rendering_uses_command_specific_summary() {
        let envelope = CommandEnvelope::failure(
            "init",
            InitCommandPayload {
                repo_name: "zazzles".into(),
                repo_root: Some(PathBuf::from("/tmp/zazzles")),
                integration_branch: "main".into(),
                integration_worktree_path: None,
                initialized: false,
                checks: Vec::new(),
            },
            FailureCategory::Auth,
            "gh auth status failed",
            "Authenticate GitHub CLI and retry.",
        );

        let rendered = envelope
            .render(CommandRenderMode::Human)
            .expect("human rendering should succeed");

        assert!(rendered.body.contains("Init failed for zazzles."));
        assert!(rendered.body.contains("Authenticate GitHub CLI and retry."));
    }
}

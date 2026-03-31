use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureCategory {
    Tooling,
    Auth,
    RepoResolution,
    Filesystem,
    Git,
    Config,
    Materialization,
}

impl FailureCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tooling => "tooling",
            Self::Auth => "auth",
            Self::RepoResolution => "repo_resolution",
            Self::Filesystem => "filesystem",
            Self::Git => "git",
            Self::Config => "config",
            Self::Materialization => "materialization",
        }
    }
}

#[derive(Debug, Error)]
pub enum ZazzlesError {
    #[error("failed to determine current directory: {source}")]
    CurrentDirectory { source: io::Error },
    #[error("HOME is not set, so ~/.zazz/config.toml cannot be resolved")]
    MissingHomeDirectory,
    #[error("failed to serialize JSON output: {source}")]
    JsonSerialization { source: serde_json::Error },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationFailure {
    pub category: FailureCategory,
    pub message: String,
    pub remediation: String,
}

impl OperationFailure {
    pub fn new(
        category: FailureCategory,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Self {
        Self {
            category,
            message: message.into(),
            remediation: remediation.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFailure<T> {
    pub payload: T,
    pub failure: OperationFailure,
}

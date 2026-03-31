use crate::errors::{FailureCategory, OperationFailure};
use crate::filesystem::write_text_atomic;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_INTEGRATION_BRANCH: &str = "main";
pub const DEFAULT_SKILLS_SYNC_SOURCE: &str = "../zazz-skills/.agents/skills";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    pub integration_branch: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoConfig {
    pub repo_name: String,
    pub integration_branch: String,
    pub integration_worktree: String,
    pub skills_sync_source: String,
}

pub fn global_config_path(home_dir: &Path) -> PathBuf {
    home_dir.join(".zazz").join("config.toml")
}

pub fn load_global_config(home_dir: &Path) -> Result<GlobalConfig, OperationFailure> {
    let path = global_config_path(home_dir);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(GlobalConfig::default());
        }
        Err(error) => {
            return Err(OperationFailure::new(
                FailureCategory::Config,
                format!("failed to read {}", path.display()),
                format!(
                    "Fix ~/.zazz/config.toml or pass --integration to bypass the unreadable global config. ({error})"
                ),
            ));
        }
    };

    toml::from_str(&contents).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Config,
            format!("failed to parse {}", path.display()),
            format!(
                "Fix ~/.zazz/config.toml or pass --integration to bypass the malformed global config. ({error})"
            ),
        )
    })
}

pub fn repo_config_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".zazz").join("config.toml")
}

pub fn write_repo_config(repo_root: &Path, config: &RepoConfig) -> Result<(), OperationFailure> {
    let path = repo_config_path(repo_root);
    let contents = toml::to_string(config).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Filesystem,
            "failed to serialize repo-local .zazz/config.toml",
            format!("Review repo-local config serialization. ({error})"),
        )
    })?;

    write_text_atomic(&path, &contents).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Filesystem,
            format!("failed to write {}", path.display()),
            format!("Ensure the repo root is writable and retry. ({error})"),
        )
    })
}

pub fn read_repo_config(repo_root: &Path) -> Result<RepoConfig, OperationFailure> {
    let path = repo_config_path(repo_root);
    let contents = fs::read_to_string(&path).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Config,
            format!("failed to read {}", path.display()),
            format!("Run `zaz init` successfully from the repo parent directory before using `zaz add`. ({error})"),
        )
    })?;

    toml::from_str(&contents).map_err(|error| {
        OperationFailure::new(
            FailureCategory::Config,
            format!("failed to parse {}", path.display()),
            format!("Fix the repo-local config or rerun `zaz init`. ({error})"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{GlobalConfig, RepoConfig};

    #[test]
    fn repo_config_serializes_expected_m1_fields() {
        let config = RepoConfig {
            repo_name: "zazzles".into(),
            integration_branch: "main".into(),
            integration_worktree: "main".into(),
            skills_sync_source: "../zazz-skills/.agents/skills".into(),
        };

        let serialized = toml::to_string(&config).expect("repo config should serialize");

        assert!(serialized.contains("repo_name = \"zazzles\""));
        assert!(serialized.contains("integration_branch = \"main\""));
        assert!(serialized.contains("integration_worktree = \"main\""));
        assert!(serialized.contains("skills_sync_source = \"../zazz-skills/.agents/skills\""));
    }

    #[test]
    fn global_config_reads_top_level_integration_branch() {
        let config: GlobalConfig =
            toml::from_str("integration_branch = \"dev\"\n").expect("global config should parse");

        assert_eq!(config.integration_branch.as_deref(), Some("dev"));
    }
}

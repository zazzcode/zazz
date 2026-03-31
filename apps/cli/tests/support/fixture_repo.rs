use super::fake_tools::{ToolPaths, create_tools};
use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use tempfile::TempDir;

pub struct CliFixture {
    _temp_dir: TempDir,
    pub parent_dir: PathBuf,
    pub remote_path: PathBuf,
    pub tools: ToolPaths,
}

impl CliFixture {
    pub fn new(branch: &str) -> Self {
        let temp_dir = TempDir::new().expect("temp dir");
        let parent_dir = temp_dir.path().to_path_buf();
        let remote_path = create_remote_fixture(&parent_dir, branch);
        let tools = create_tools(&parent_dir);

        Self {
            _temp_dir: temp_dir,
            parent_dir,
            remote_path,
            tools,
        }
    }

    pub fn repo_root(&self, repo_name: &str) -> PathBuf {
        self.parent_dir.join(repo_name)
    }

    pub fn command(&self, cwd: &Path) -> Command {
        let mut command = Command::cargo_bin("zaz").expect("binary should build");
        command.current_dir(cwd);
        command.env("ZAZ_GIT_BIN", &self.tools.git_wrapper);
        command.env("ZAZ_GH_BIN", &self.tools.gh_wrapper);
        command.env("ZAZ_TEST_GH_REMOTE_PATH", &self.remote_path);
        command
    }
}

fn create_remote_fixture(parent: &Path, branch: &str) -> PathBuf {
    let source = parent.join(format!("remote-source-{branch}"));
    let bare = parent.join(format!("remote-{branch}.git"));
    run_git(
        parent,
        [
            "init",
            source.to_string_lossy().as_ref(),
            "--initial-branch",
            "main",
        ],
    );
    run_git(&source, ["config", "user.email", "tests@example.com"]);
    run_git(&source, ["config", "user.name", "Test Runner"]);
    fs::write(source.join("README.md"), "# sample\n").expect("write readme");
    run_git(&source, ["add", "README.md"]);
    run_git(&source, ["commit", "-m", "Initial commit"]);
    if branch != "main" {
        run_git(&source, ["checkout", "-b", branch]);
    }
    run_git(
        parent,
        [
            "clone",
            "--bare",
            source.to_string_lossy().as_ref(),
            bare.to_string_lossy().as_ref(),
        ],
    );
    bare
}

fn run_git<I, S>(cwd: &Path, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let status = ProcessCommand::new(find_real_git())
        .current_dir(cwd)
        .args(args.into_iter().map(|arg| arg.as_ref().to_string()))
        .status()
        .expect("git command should run");
    assert!(status.success(), "git command should succeed");
}

fn find_real_git() -> PathBuf {
    let output = ProcessCommand::new("which")
        .arg("git")
        .output()
        .expect("which git should run");
    assert!(output.status.success(), "which git should succeed");
    PathBuf::from(String::from_utf8_lossy(&output.stdout).trim())
}

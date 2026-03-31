use crate::commands::InitRequest;
use crate::init::{build_manifest, execute};
use crate::state::UntrackedEntryKind;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

#[test]
fn build_manifest_sorts_entries_and_marks_seeded_items_optional() {
    let manifest = build_manifest(
        "dev",
        vec![
            crate::git::DiscoveredUntrackedPath {
                path: "z-dir".into(),
                kind: UntrackedEntryKind::Directory,
            },
            crate::git::DiscoveredUntrackedPath {
                path: ".env".into(),
                kind: UntrackedEntryKind::File,
            },
            crate::git::DiscoveredUntrackedPath {
                path: "z-dir/nested.env".into(),
                kind: UntrackedEntryKind::File,
            },
            crate::git::DiscoveredUntrackedPath {
                path: ".env".into(),
                kind: UntrackedEntryKind::File,
            },
        ],
    );

    assert_eq!(manifest.source_worktree, "dev");
    assert_eq!(manifest.entries.len(), 2);
    assert_eq!(manifest.entries[0].path, ".env");
    assert_eq!(manifest.entries[1].path, "z-dir");
    assert!(
        !manifest
            .entries
            .iter()
            .any(|entry| entry.path == "z-dir/nested.env")
    );
    assert!(manifest.entries.iter().all(|entry| !entry.required));
}

#[test]
fn init_success_creates_repo_layout_and_state_files() {
    let _guard = env_lock().lock().expect("lock env");
    let temp_dir = TempDir::new().expect("temp dir");
    let remote = create_remote_fixture(temp_dir.path(), "main");
    let git_wrapper = create_git_wrapper(temp_dir.path());
    let gh_wrapper = create_gh_wrapper(temp_dir.path(), &remote, true, true);

    let previous_git = env::var_os("ZAZ_GIT_BIN");
    let previous_gh = env::var_os("ZAZ_GH_BIN");
    unsafe {
        env::set_var("ZAZ_GIT_BIN", &git_wrapper);
        env::set_var("ZAZ_GH_BIN", &gh_wrapper);
    }

    let outcome = execute(InitRequest {
        repo_name: "sample-repo".into(),
        integration_branch_override: Some("main".into()),
        cwd: temp_dir.path().to_path_buf(),
        home_dir: temp_dir.path().to_path_buf(),
    })
    .expect("init should succeed");

    restore_env("ZAZ_GIT_BIN", previous_git);
    restore_env("ZAZ_GH_BIN", previous_gh);

    let repo_root = temp_dir.path().join("sample-repo");
    assert!(repo_root.join(".bare").exists());
    assert!(repo_root.join("main").exists());
    assert!(repo_root.join(".zazz/config.toml").exists());
    assert!(repo_root.join(".zazz/graph.json").exists());
    assert!(repo_root.join(".zazz/worktrees.json").exists());
    assert!(repo_root.join(".zazz/untracked-files.json").exists());
    assert!(repo_root.join(".zazz/conflicts").exists());
    assert!(repo_root.join(".zazz/locks").exists());
    assert!(outcome.initialized);
    assert_eq!(outcome.integration_branch, "main");

    let manifest =
        fs::read_to_string(repo_root.join(".zazz/untracked-files.json")).expect("manifest exists");
    assert!(manifest.contains("\"sourceWorktree\": \"main\""));
    assert!(manifest.contains("\"path\": \".env.local\""));
    assert!(manifest.contains("\"path\": \"tmp-dir\""));

    let config =
        fs::read_to_string(repo_root.join(".zazz/config.toml")).expect("config should exist");
    assert!(config.contains("repo_name = \"sample-repo\""));
    assert!(config.contains("integration_branch = \"main\""));
    assert!(config.contains("skills_sync_source = \"../zazz-skills/.agents/skills\""));

    let exclude =
        fs::read_to_string(repo_root.join(".bare/info/exclude")).expect("exclude should exist");
    assert!(exclude.contains(".claude/skills/"));
}

#[test]
fn init_auth_failure_leaves_no_partial_repo_root() {
    let _guard = env_lock().lock().expect("lock env");
    let temp_dir = TempDir::new().expect("temp dir");
    let remote = create_remote_fixture(temp_dir.path(), "main");
    let gh_wrapper = create_gh_wrapper(temp_dir.path(), &remote, true, false);

    let previous_gh = env::var_os("ZAZ_GH_BIN");
    unsafe {
        env::set_var("ZAZ_GH_BIN", &gh_wrapper);
    }

    let failure = execute(InitRequest {
        repo_name: "sample-repo".into(),
        integration_branch_override: Some("main".into()),
        cwd: temp_dir.path().to_path_buf(),
        home_dir: temp_dir.path().to_path_buf(),
    })
    .expect_err("init should fail without gh auth");

    restore_env("ZAZ_GH_BIN", previous_gh);

    assert_eq!(
        failure.failure.category,
        crate::errors::FailureCategory::Auth
    );
    assert!(!temp_dir.path().join("sample-repo").exists());
}

#[test]
fn init_repo_resolution_failure_leaves_no_partial_repo_root() {
    let _guard = env_lock().lock().expect("lock env");
    let temp_dir = TempDir::new().expect("temp dir");
    let remote = create_remote_fixture(temp_dir.path(), "main");
    let gh_wrapper = create_gh_wrapper(temp_dir.path(), &remote, false, true);

    let previous_gh = env::var_os("ZAZ_GH_BIN");
    unsafe {
        env::set_var("ZAZ_GH_BIN", &gh_wrapper);
    }

    let failure = execute(InitRequest {
        repo_name: "sample-repo".into(),
        integration_branch_override: Some("main".into()),
        cwd: temp_dir.path().to_path_buf(),
        home_dir: temp_dir.path().to_path_buf(),
    })
    .expect_err("init should fail when repo does not resolve");

    restore_env("ZAZ_GH_BIN", previous_gh);

    assert_eq!(
        failure.failure.category,
        crate::errors::FailureCategory::RepoResolution
    );
    assert!(!temp_dir.path().join("sample-repo").exists());
}

#[test]
fn init_prefers_global_config_when_no_flag_is_provided() {
    let _guard = env_lock().lock().expect("lock env");
    let temp_dir = TempDir::new().expect("temp dir");
    fs::create_dir_all(temp_dir.path().join(".zazz")).expect("create home config dir");
    fs::write(
        temp_dir.path().join(".zazz/config.toml"),
        "integration_branch = \"dev\"\n",
    )
    .expect("write global config");
    let remote = create_remote_fixture(temp_dir.path(), "dev");
    let git_wrapper = create_git_wrapper(temp_dir.path());
    let gh_wrapper = create_gh_wrapper(temp_dir.path(), &remote, true, true);

    let previous_git = env::var_os("ZAZ_GIT_BIN");
    let previous_gh = env::var_os("ZAZ_GH_BIN");
    unsafe {
        env::set_var("ZAZ_GIT_BIN", &git_wrapper);
        env::set_var("ZAZ_GH_BIN", &gh_wrapper);
    }

    let outcome = execute(InitRequest {
        repo_name: "sample-repo".into(),
        integration_branch_override: None,
        cwd: temp_dir.path().to_path_buf(),
        home_dir: temp_dir.path().to_path_buf(),
    })
    .expect("init should succeed");

    restore_env("ZAZ_GIT_BIN", previous_git);
    restore_env("ZAZ_GH_BIN", previous_gh);

    assert_eq!(outcome.integration_branch, "dev");
    assert!(temp_dir.path().join("sample-repo/dev").exists());
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

fn create_git_wrapper(parent: &Path) -> PathBuf {
    let path = parent.join("fake-git.sh");
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"status\" ] && [ \"$2\" = \"--porcelain\" ] && [ \"$3\" = \"--untracked-files=normal\" ]; then\n  printf '?? .env.local\\n?? tmp-dir/\\n'\n  exit 0\nfi\nexec {} \"$@\"\n",
        find_real_git().display()
    );
    fs::write(&path, script).expect("write fake git");
    let mut permissions = fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("chmod");
    path
}

fn create_gh_wrapper(parent: &Path, remote: &Path, repo_exists: bool, auth_ok: bool) -> PathBuf {
    let path = parent.join("fake-gh.sh");
    let repo_json = if repo_exists {
        format!(
            "{{\"nameWithOwner\":\"example/sample-repo\",\"url\":\"{}\"}}",
            remote.display()
        )
    } else {
        String::new()
    };
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  printf 'gh version 2.0.0\\n'\n  exit 0\nfi\nif [ \"$1\" = \"auth\" ] && [ \"$2\" = \"status\" ]; then\n  {auth_block}\nfi\nif [ \"$1\" = \"repo\" ] && [ \"$2\" = \"view\" ]; then\n  {repo_block}\nfi\nprintf 'unsupported gh invocation: %s\\n' \"$*\" >&2\nexit 1\n",
        auth_block = if auth_ok {
            "printf 'Logged in\\n'; exit 0"
        } else {
            "printf 'not logged in\\n' >&2; exit 1"
        },
        repo_block = if repo_exists {
            format!("printf '%s\\n' '{}'; exit 0", repo_json)
        } else {
            "printf 'repo not found\\n' >&2; exit 1".to_string()
        }
    );
    fs::write(&path, script).expect("write fake gh");
    let mut permissions = fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("chmod");
    path
}

fn run_git<I, S>(cwd: &Path, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let status = Command::new(find_real_git())
        .current_dir(cwd)
        .args(args.into_iter().map(|arg| arg.as_ref().to_string()))
        .status()
        .expect("git command should run");
    assert!(status.success(), "git command should succeed");
}

fn find_real_git() -> PathBuf {
    let output = Command::new("which")
        .arg("git")
        .output()
        .expect("which git should run");
    assert!(output.status.success(), "which git should succeed");
    PathBuf::from(String::from_utf8_lossy(&output.stdout).trim())
}

fn restore_env(key: &str, value: Option<std::ffi::OsString>) {
    match value {
        Some(value) => unsafe { env::set_var(key, value) },
        None => unsafe { env::remove_var(key) },
    }
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

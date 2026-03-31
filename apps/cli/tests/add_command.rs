mod support;

use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use support::fixture_repo::CliFixture;

#[test]
fn add_human_success_creates_worktree_and_materializes_manifest_entries() {
    let fixture = CliFixture::new("main");
    init_repo(&fixture);

    let repo_root = fixture.repo_root("sample-repo");
    let integration_worktree = repo_root.join("main");
    fs::write(integration_worktree.join(".env.local"), "TOKEN=abc\n").expect("write env");
    fs::create_dir_all(integration_worktree.join("tmp-dir/nested")).expect("create dir");
    fs::write(
        integration_worktree.join("tmp-dir/nested/file.txt"),
        "nested\n",
    )
    .expect("write nested");
    fs::write(integration_worktree.join("scratch.txt"), "skip\n").expect("write scratch");
    fs::write(
        repo_root.join(".zazz/untracked-files.json"),
        r#"{
  "version": 1,
  "sourceWorktree": "main",
  "entries": [
    { "path": ".env.local", "kind": "file", "required": false },
    { "path": "tmp-dir", "kind": "directory", "required": false }
  ]
}
"#,
    )
    .expect("rewrite manifest");

    fixture
        .command(&repo_root)
        .args(["add", "feature-a"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created worktree feature-a"))
        .stdout(predicate::str::contains("skipped missing skills source"));

    let worktree = repo_root.join("feature-a");
    assert!(worktree.join(".env.local").exists());
    assert!(worktree.join("tmp-dir/nested/file.txt").exists());
    assert!(!worktree.join("scratch.txt").exists());
}

#[test]
fn add_json_success_reports_worktree_and_materialization() {
    let fixture = CliFixture::new("main");
    init_repo(&fixture);

    let repo_root = fixture.repo_root("sample-repo");
    fs::write(
        repo_root.join(".zazz/untracked-files.json"),
        r#"{
  "version": 1,
  "sourceWorktree": "main",
  "entries": []
}
"#,
    )
    .expect("rewrite manifest");

    let output = fixture
        .command(&repo_root)
        .args(["add", "feature-json", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("valid json");

    assert_eq!(json["command"], "add");
    assert_eq!(json["success"], true);
    assert_eq!(json["branch"], "feature-json");
    assert_eq!(json["materialized"], false);
    assert!(json["worktreePath"].as_str().is_some());
}

#[test]
fn add_does_not_auto_resync_existing_worktrees_when_integration_untracked_files_change() {
    let fixture = CliFixture::new("main");
    init_repo(&fixture);

    let repo_root = fixture.repo_root("sample-repo");
    let integration_worktree = repo_root.join("main");
    fs::write(
        repo_root.join(".zazz/untracked-files.json"),
        r#"{
  "version": 1,
  "sourceWorktree": "main",
  "entries": [
    { "path": ".env.local", "kind": "file", "required": false }
  ]
}
"#,
    )
    .expect("rewrite manifest");
    fs::write(integration_worktree.join(".env.local"), "TOKEN=one\n").expect("write v1");

    fixture
        .command(&repo_root)
        .args(["add", "feature-old"])
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(repo_root.join("feature-old/.env.local")).expect("old copy exists"),
        "TOKEN=one\n"
    );

    fs::write(integration_worktree.join(".env.local"), "TOKEN=two\n").expect("write v2");

    fixture
        .command(&repo_root)
        .args(["add", "feature-new"])
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(repo_root.join("feature-old/.env.local")).expect("old copy remains"),
        "TOKEN=one\n"
    );
    assert_eq!(
        fs::read_to_string(repo_root.join("feature-new/.env.local")).expect("new copy updated"),
        "TOKEN=two\n"
    );
}

#[test]
fn add_json_failure_reports_fetch_errors_without_partial_state() {
    let fixture = CliFixture::new("main");
    init_repo(&fixture);
    let repo_root = fixture.repo_root("sample-repo");

    let output = fixture
        .command(&repo_root)
        .env("ZAZ_TEST_FAIL_FETCH", "1")
        .args(["add", "feature-b", "--json"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("valid json");

    assert_eq!(json["command"], "add");
    assert_eq!(json["success"], false);
    assert_eq!(json["failure"]["category"], "git");
    assert!(!repo_root.join("feature-b").exists());
}

#[test]
fn add_human_failure_when_target_directory_exists_is_clear() {
    let fixture = CliFixture::new("main");
    init_repo(&fixture);
    let repo_root = fixture.repo_root("sample-repo");
    fs::create_dir_all(repo_root.join("feature-c")).expect("create conflicting dir");

    fixture
        .command(&repo_root)
        .args(["add", "feature-c"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "target worktree directory already exists",
        ));
}

fn init_repo(fixture: &CliFixture) {
    fixture
        .command(&fixture.parent_dir)
        .args(["init", "sample-repo", "--integration", "main"])
        .assert()
        .success();
}

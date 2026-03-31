mod support;

use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use support::fixture_repo::CliFixture;

#[test]
fn init_human_success_creates_repo_root() {
    let fixture = CliFixture::new("main");

    fixture
        .command(&fixture.parent_dir)
        .args(["init", "sample-repo", "--integration", "main"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized sample-repo"));

    let repo_root = fixture.repo_root("sample-repo");
    assert!(repo_root.join(".bare").exists());
    assert!(repo_root.join("main").exists());
    assert!(repo_root.join(".zazz/config.toml").exists());
}

#[test]
fn init_human_success_from_nested_parent_directory() {
    let fixture = CliFixture::new("main");
    let nested_parent = fixture.parent_dir.join("smoke-root").join("nested-parent");
    fs::create_dir_all(&nested_parent).expect("create nested parent");

    fixture
        .command(&nested_parent)
        .args(["init", "nested-repo", "--integration", "main"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized nested-repo"));

    let repo_root = nested_parent.join("nested-repo");
    assert!(repo_root.join(".bare").exists());
    assert!(repo_root.join("main").exists());
    assert!(repo_root.join(".zazz/config.toml").exists());
}

#[test]
fn init_json_success_reports_paths_and_checks() {
    let fixture = CliFixture::new("main");

    let output = fixture
        .command(&fixture.parent_dir)
        .args(["init", "sample-repo", "--integration", "main", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("valid json");

    assert_eq!(json["command"], "init");
    assert_eq!(json["success"], true);
    assert_eq!(json["repoName"], "sample-repo");
    assert_eq!(json["integrationBranch"], "main");
    assert_eq!(json["initialized"], true);
    assert!(json["checks"].is_array());
}

#[test]
fn init_json_failure_reports_auth_problems() {
    let fixture = CliFixture::new("main");

    let output = fixture
        .command(&fixture.parent_dir)
        .env("ZAZ_TEST_GH_AUTH", "fail")
        .args(["init", "sample-repo", "--json"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("valid json");

    assert_eq!(json["command"], "init");
    assert_eq!(json["success"], false);
    assert_eq!(json["failure"]["category"], "auth");
    assert!(
        !fixture.repo_root("sample-repo").exists(),
        "failed init should not leave a partial repo root"
    );
}

#[test]
fn init_json_failure_reports_repo_resolution_problems() {
    let fixture = CliFixture::new("main");

    let output = fixture
        .command(&fixture.parent_dir)
        .env("ZAZ_TEST_GH_REPO", "missing")
        .args(["init", "sample-repo", "--json"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("valid json");

    assert_eq!(json["command"], "init");
    assert_eq!(json["success"], false);
    assert_eq!(json["failure"]["category"], "repo_resolution");
    assert!(
        !fixture.repo_root("sample-repo").exists(),
        "failed init should not leave a partial repo root"
    );
}

#[test]
fn init_human_failure_when_gh_is_missing_is_actionable() {
    let fixture = CliFixture::new("main");

    fixture
        .command(&fixture.parent_dir)
        .env("ZAZ_GH_BIN", fixture.parent_dir.join("missing-gh"))
        .args(["init", "sample-repo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "GitHub CLI is not installed or not callable",
        ));
}

#[test]
fn init_rejects_existing_target_directory() {
    let fixture = CliFixture::new("main");
    fs::create_dir_all(fixture.repo_root("sample-repo")).expect("create conflicting dir");

    fixture
        .command(&fixture.parent_dir)
        .args(["init", "sample-repo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("target directory already exists"));
}

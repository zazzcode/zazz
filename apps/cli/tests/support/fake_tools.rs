use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct ToolPaths {
    pub git_wrapper: PathBuf,
    pub gh_wrapper: PathBuf,
}

pub fn create_tools(parent: &Path) -> ToolPaths {
    ToolPaths {
        git_wrapper: create_git_wrapper(parent),
        gh_wrapper: create_gh_wrapper(parent),
    }
}

fn create_git_wrapper(parent: &Path) -> PathBuf {
    let path = parent.join("fake-git.sh");
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"status\" ] && [ \"$2\" = \"--porcelain\" ] && [ \"$3\" = \"--untracked-files=normal\" ] && [ -n \"$ZAZ_TEST_STATUS_OUTPUT\" ]; then\n  printf '%s' \"$ZAZ_TEST_STATUS_OUTPUT\"\n  exit 0\nfi\ncase \"$1\" in\n  --git-dir=*)\n    if [ \"$2\" = \"fetch\" ] && [ \"$ZAZ_TEST_FAIL_FETCH\" = \"1\" ]; then\n      printf 'fetch failed\\n' >&2\n      exit 1\n    fi\n    ;;\nesac\nif [ \"$1\" = \"pull\" ] && [ \"$2\" = \"--ff-only\" ] && [ \"$ZAZ_TEST_FAIL_PULL\" = \"1\" ]; then\n  printf 'pull failed\\n' >&2\n  exit 1\nfi\nexec {} \"$@\"\n",
        find_real_git().display()
    );
    write_executable(&path, &script);
    path
}

fn create_gh_wrapper(parent: &Path) -> PathBuf {
    let path = parent.join("fake-gh.sh");
    let script = "#!/bin/sh
if [ \"$1\" = \"--version\" ]; then
  printf 'gh version 2.0.0\n'
  exit 0
fi
if [ \"$1\" = \"auth\" ] && [ \"$2\" = \"status\" ]; then
  if [ \"$ZAZ_TEST_GH_AUTH\" = \"fail\" ]; then
    printf 'not logged in\n' >&2
    exit 1
  fi
  printf 'Logged in\n'
  exit 0
fi
if [ \"$1\" = \"repo\" ] && [ \"$2\" = \"view\" ]; then
  if [ \"$ZAZ_TEST_GH_REPO\" = \"missing\" ]; then
    printf 'repo not found\n' >&2
    exit 1
  fi
  printf '{\"nameWithOwner\":\"example/%s\",\"url\":\"%s\"}\n' \"$3\" \"$ZAZ_TEST_GH_REMOTE_PATH\"
  exit 0
fi
printf 'unsupported gh invocation: %s\n' \"$*\" >&2
exit 1
";
    write_executable(&path, script);
    path
}

fn write_executable(path: &Path, script: &str) {
    fs::write(path, script).expect("write wrapper");
    let mut permissions = fs::metadata(path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("chmod");
}

fn find_real_git() -> PathBuf {
    let output = Command::new("which")
        .arg("git")
        .output()
        .expect("which git should run");
    assert!(output.status.success(), "which git should succeed");
    PathBuf::from(String::from_utf8_lossy(&output.stdout).trim())
}

use std::env;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

#[derive(Debug, Clone)]
pub struct ToolNames {
    pub git: OsString,
    pub gh: OsString,
}

impl ToolNames {
    pub fn from_env() -> Self {
        Self {
            git: env::var_os("ZAZ_GIT_BIN").unwrap_or_else(|| OsString::from("git")),
            gh: env::var_os("ZAZ_GH_BIN").unwrap_or_else(|| OsString::from("gh")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub cwd: Option<PathBuf>,
}

impl CommandSpec {
    pub fn display(&self) -> String {
        let mut parts = vec![self.program.to_string_lossy().into_owned()];
        parts.extend(
            self.args
                .iter()
                .map(|value| value.to_string_lossy().into_owned()),
        );
        parts.join(" ")
    }
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

pub fn run(spec: &CommandSpec) -> io::Result<CommandOutput> {
    let mut command = Command::new(&spec.program);
    command.args(&spec.args);
    if let Some(cwd) = spec.cwd.as_ref() {
        command.current_dir(cwd);
    }

    let output = command.output()?;
    Ok(CommandOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

pub fn path_to_os_string(path: &Path) -> OsString {
    path.as_os_str().to_os_string()
}

use crate::add;
use crate::errors::ZazzlesError;
use crate::init;
use crate::output::{CommandEnvelope, CommandRenderMode, RenderedCommandOutput};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitRequest {
    pub repo_name: String,
    pub integration_branch_override: Option<String>,
    pub cwd: PathBuf,
    pub home_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddRequest {
    pub branch_name: String,
    pub cwd: PathBuf,
    pub home_dir: PathBuf,
}

pub fn dispatch_init(
    request: InitRequest,
    render_mode: CommandRenderMode,
) -> Result<RenderedCommandOutput, ZazzlesError> {
    let envelope = match init::execute(request) {
        Ok(payload) => CommandEnvelope::success("init", payload),
        Err(error) => {
            let error = *error;
            CommandEnvelope::failure(
                "init",
                error.payload,
                error.failure.category,
                error.failure.message,
                error.failure.remediation,
            )
        }
    };

    envelope.render(render_mode)
}

pub fn dispatch_add(
    request: AddRequest,
    render_mode: CommandRenderMode,
) -> Result<RenderedCommandOutput, ZazzlesError> {
    let envelope = match add::execute(request) {
        Ok(payload) => CommandEnvelope::success("add", payload),
        Err(error) => {
            let error = *error;
            CommandEnvelope::failure(
                "add",
                error.payload,
                error.failure.category,
                error.failure.message,
                error.failure.remediation,
            )
        }
    };

    envelope.render(render_mode)
}

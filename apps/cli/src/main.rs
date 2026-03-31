mod cli;

use clap::Parser;
use std::env;
use std::process::ExitCode;
use zazzles_core::commands::{AddRequest, InitRequest};
use zazzles_core::errors::ZazzlesError;
use zazzles_core::output::{CommandRenderMode, RenderedCommandOutput};

fn main() -> ExitCode {
    match run() {
        Ok(output) => {
            print_output(&output);
            if output.success {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<RenderedCommandOutput, ZazzlesError> {
    let args = cli::Cli::parse();
    let cwd = env::current_dir().map_err(|source| ZazzlesError::CurrentDirectory { source })?;
    let home_dir = env::var_os("HOME")
        .map(Into::into)
        .ok_or(ZazzlesError::MissingHomeDirectory)?;

    match args.command {
        cli::Commands::Init(command) => zazzles_core::commands::dispatch_init(
            InitRequest {
                repo_name: command.repo_name,
                integration_branch_override: command.integration_branch,
                cwd,
                home_dir,
            },
            render_mode(command.json),
        ),
        cli::Commands::Add(command) => zazzles_core::commands::dispatch_add(
            AddRequest {
                branch_name: command.name,
                cwd,
                home_dir,
            },
            render_mode(command.json),
        ),
    }
}

fn render_mode(json: bool) -> CommandRenderMode {
    if json {
        CommandRenderMode::Json
    } else {
        CommandRenderMode::Human
    }
}

fn print_output(output: &RenderedCommandOutput) {
    if output.success {
        println!("{}", output.body);
    } else {
        eprintln!("{}", output.body);
    }
}

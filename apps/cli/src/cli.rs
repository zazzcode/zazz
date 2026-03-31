use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "zaz", version, about = "Worktree-native orchestration CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init(InitCommand),
    Add(AddCommand),
}

#[derive(Debug, Args)]
pub struct InitCommand {
    pub repo_name: String,
    #[arg(long = "integration", short = 'i')]
    pub integration_branch: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct AddCommand {
    pub name: String,
    #[arg(long)]
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use super::{Cli, Commands};
    use clap::Parser;

    #[test]
    fn parses_init_with_short_integration_flag() {
        let cli = Cli::parse_from(["zaz", "init", "zazzles", "-i", "dev"]);

        match cli.command {
            Commands::Init(command) => {
                assert_eq!(command.repo_name, "zazzles");
                assert_eq!(command.integration_branch.as_deref(), Some("dev"));
                assert!(!command.json);
            }
            Commands::Add(_) => panic!("expected init command"),
        }
    }

    #[test]
    fn parses_add_with_json_output() {
        let cli = Cli::parse_from(["zaz", "add", "feature-a", "--json"]);

        match cli.command {
            Commands::Add(command) => {
                assert_eq!(command.name, "feature-a");
                assert!(command.json);
            }
            Commands::Init(_) => panic!("expected add command"),
        }
    }

    #[test]
    fn rejects_missing_repo_name() {
        let error =
            Cli::try_parse_from(["zaz", "init"]).expect_err("init should require repo name");
        let rendered = error.to_string();

        assert!(rendered.contains("<REPO_NAME>"));
    }

    #[test]
    fn rejects_unsupported_add_flags() {
        let error = Cli::try_parse_from(["zaz", "add", "feature-a", "--integration", "main"])
            .expect_err("add should reject integration flag");
        let rendered = error.to_string();

        assert!(rendered.contains("--integration"));
    }
}

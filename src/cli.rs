use clap::{Parser, Subcommand};
use std::ffi::OsString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandAction {
    RunTui,
    Update,
    Check,
}

#[derive(Parser, Debug)]
#[command(name = "norenwake", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Update,
    Check,
}

pub fn parse_command<I, T>(args: I) -> Result<CommandAction, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::try_parse_from(args)?;
    Ok(match cli.command {
        Some(Commands::Update) => CommandAction::Update,
        Some(Commands::Check) => CommandAction::Check,
        None => CommandAction::RunTui,
    })
}

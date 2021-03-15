use std::path::PathBuf;

use clap::Clap;

#[derive(Clap)]
#[clap(version = "1.0", author = "Angel of Sol")]
pub struct Opts {
    #[clap(short, long)]
    pub gui: bool,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    /// Creates a new database in the current working directory
    Init { path: Option<PathBuf> },
    /// Add a new file to the database
    Add {
        path: PathBuf,
        #[clap(default_value = "0")]
        dir: usize,
    },
    /// Updates file explorer context menus
    Contextual {
        #[clap(subcommand)]
        subcmd: ContextualSubCommand,
    },
    /// Resets the configuration to the default values
    ResetConfig,
}

#[derive(Clap)]
pub enum ContextualSubCommand {
    /// Installs context menu handlers to the local file explorer
    Install,
    /// Removes context menu handlers from the local file explorer
    Remove,
}

use std::io::stdin;

use clap::Clap;
use config::Config;

mod cli;
mod config;
mod contextual;
mod project_dir;

fn main() {
    let mut config: Config = if let Some(config) = Config::load() {
        config
    } else {
        let config = Config::default();
        config.save().unwrap();
        config
    };

    let opts = cli::Opts::parse();

    match opts.subcmd {
        cli::SubCommand::Init { path } => {
            let path = path.or_else(|| std::env::current_dir().ok()).unwrap();
            if !config.data_dirs.contains(&path) {
                config.data_dirs.push(path);
                config.save().unwrap();
            }

            println!("{:?}", config.data_dirs);
            let mut x = String::new();
            stdin().read_line(&mut x).unwrap();
        }
        cli::SubCommand::Contextual { subcmd } => match subcmd {
            cli::ContextualSubCommand::Install => {
                contextual::install().unwrap();
            }
            cli::ContextualSubCommand::Remove => {
                contextual::remove().unwrap();
            }
        },
        cli::SubCommand::Add { path } => {
            println!("Adding: {:?}", path);
            let mut x = String::new();
            stdin().read_line(&mut x).unwrap();
        }
    }
}

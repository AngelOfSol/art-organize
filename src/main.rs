use std::io::stdin;

use backend::Backend;
use clap::Clap;
use config::Config;

mod backend;
mod cli;
mod config;
mod model;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config: Config = if let Ok(config) = Config::load() {
        config
    } else {
        let config = Config::default();
        config.save().unwrap();
        config
    };

    let opts = cli::Opts::parse();

    match opts.subcmd {
        cli::SubCommand::Init { path } => {
            let root = path.unwrap_or(std::env::current_dir()?);
            if !config.data_dirs.contains(&root) {
                config.data_dirs.push(root.clone());
                config.save().unwrap();
            }

            let _ = Backend::init_at_path(root).await?;
        }
        cli::SubCommand::Contextual { subcmd } => match subcmd {
            cli::ContextualSubCommand::Install => {
                contextual::install()?;
            }
            cli::ContextualSubCommand::Remove => {
                contextual::remove()?;
            }
        },
        cli::SubCommand::Add { path, dir } => {
            let root = config.data_dirs[dir].clone();
            let mut backend = Backend::from_path(root).await?;
            backend.add_file(path).await?;
            for piece in backend.query_pieces().await? {
                dbg!(piece);
            }
            let mut x = String::new();
            stdin().read_line(&mut x)?;
        }
        cli::SubCommand::ResetConfig => {
            config = Config::default();
            config.save()?;
        }
    }

    Ok(())
}

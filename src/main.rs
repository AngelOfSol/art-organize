use std::sync::{Arc, RwLock};

use app::{
    actor::{AppActor, AppState, Inner},
    App,
};
use backend::Backend;
use clap::Clap;
use config::Config;
use gui::{run_event_loop, GuiContext};
use ipc::start_server;
use tokio::runtime::{Builder, Handle};
use winit::event_loop::EventLoop;

mod app;
mod backend;
mod cli;
mod config;
mod consts;
mod gui;
mod loaders;
mod raw_image;
mod style;

fn main() -> anyhow::Result<()> {
    let mut runtime = Builder::new_multi_thread().enable_all().build()?;

    runtime.block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    let mut config: Config = if let Ok(config) = Config::load() {
        config
    } else {
        let config = Config::default();
        config.save().unwrap();
        config
    };

    let opts = cli::Opts::parse();

    let client = ipc::try_connect();

    if let Some(client) = client {
        return client.send(opts.subcmd);
    }

    match opts.subcmd {
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
        }
        cli::SubCommand::Init { path } => {
            let root = path.unwrap_or(std::env::current_dir()?);
            if !config.data_dirs.contains(&root) {
                config.data_dirs.push(root.clone());
                config.save().unwrap();
            }

            let _ = Backend::init_at_path(root).await?;
        }
        cli::SubCommand::ResetConfig => {
            config = Config::default();
            config.save()?;
        }
        cli::SubCommand::Gui => {
            let root = config.data_dirs[0].clone();
            let backend = Backend::from_path(root).await?;
            let event_loop = EventLoop::new();
            let app = App {
                actor: Arc::new(AppActor(RwLock::new(Inner {
                    handle: Handle::current(),
                    backend,
                    ipc: start_server()?,
                    state: AppState::None,
                    image_cache: Default::default(),
                }))),
            };
            let gui = GuiContext::create(&event_loop).await?;

            run_event_loop(event_loop, gui, app);
        }
    }

    Ok(())
}

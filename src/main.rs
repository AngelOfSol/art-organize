#![windows_subsystem = "windows"]

use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use app::{
    actor::{AppActor, Inner},
    gui_state::GuiState,
    App,
};
use backend::DbBackend;
use clap::Clap;
use config::Config;
use gui::{run_event_loop, GuiContext};
use ipc::start_server;
use tokio::{
    runtime::{Builder, Handle},
    sync::mpsc,
};
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
mod undo;

fn main() -> anyhow::Result<()> {
    let runtime = Builder::new_multi_thread().enable_all().build()?;

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
            let mut backend = DbBackend::from_path(root).await?;
            backend.add_file(path).await?;
        }
        cli::SubCommand::Init { path } => {
            let root = path.unwrap_or(std::env::current_dir()?);
            if !config.data_dirs.contains(&root) {
                config.data_dirs.push(root.clone());
                config.save().unwrap();
            }

            let _ = DbBackend::init_at_path(root).await?;
        }
        cli::SubCommand::ResetConfig => {
            config = Config::default();
            config.save()?;
        }
        cli::SubCommand::Gui => {
            let root = config.data_dirs[0].clone();
            let backend = DbBackend::from_path(root).await?;
            let event_loop = EventLoop::new();

            let (tx, rx) = mpsc::channel(4);

            let app = App {
                actor: Arc::new(AppActor(RwLock::new(Inner {
                    handle: Handle::current(),
                    db: backend,
                    ipc: start_server()?,
                    image_cache: Default::default(),
                    outgoing_images: tx,
                    gui_state: GuiState::default(),
                }))),
                incoming_images: rx,
                images: BTreeMap::new(),
            };
            let gui = GuiContext::create(&event_loop).await?;

            run_event_loop(event_loop, gui, app);
        }
    }

    Ok(())
}

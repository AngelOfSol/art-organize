// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(btree_retain)]

use std::sync::{Arc, RwLock};

use app::{
    gui_state::{start_gui_task, GuiState},
    App,
};
use backend::{actor::start_db_task, DbBackend};
use clap::Clap;
use cli::SubCommand;
use config::Config;
use gui::{run_event_loop, GuiContext};
use ipc::start_server;
use tokio::{runtime::Builder, sync::mpsc};
use winit::event_loop::EventLoop;

mod app;
mod backend;
mod cli;
mod config;
mod consts;
mod gui;
mod layout;
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
        cli::SubCommand::Init { path } => {
            let root = path.unwrap_or(std::env::current_dir()?);

            config.default_dir = Some(root.clone());

            config.save().unwrap();

            let _ = DbBackend::init_at_path(root).await?;
        }
        cli::SubCommand::ResetConfig => {
            config = Config::default();
            config.save()?;
        }
        cli::SubCommand::Gui => {
            let (outgoing_images, rx) = mpsc::unbounded_channel();
            let (outgoing_files, incoming_files) = std::sync::mpsc::channel();
            let root = config.default_dir.unwrap_or(std::env::current_dir()?);

            let db = Arc::new(RwLock::new(DbBackend::from_path(root).await?));
            let gui_state = Arc::new(RwLock::new(GuiState::default()));

            let _ipc = start_server::<SubCommand>()?;

            let db_handle = start_db_task(db);
            let gui_handle = start_gui_task(
                db_handle.clone(),
                gui_state.clone(),
                outgoing_images,
                incoming_files,
            );

            let event_loop = EventLoop::new();

            let app = App {
                incoming_images: rx,
                gui_handle,
                gui_state,
            };
            let gui = GuiContext::create(&event_loop, outgoing_files).await?;

            run_event_loop(event_loop, gui, app);
        }
    }

    Ok(())
}

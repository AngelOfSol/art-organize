// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(btree_retain)]

use std::{
    path::PathBuf,
    sync::{mpsc, Arc, RwLock},
};

use anyhow::bail;
use app::{
    gui_state::{start_gui_task, GuiState},
    App,
};
use backend::{actor::start_db_task, DbBackend};
use clap::Clap;
use cli::SubCommand;
use config::Config;
use gui::{run_event_loop, GuiContext};
use rfd::{AsyncFileDialog, AsyncMessageDialog, MessageButtons, MessageLevel};
use tokio::runtime::Builder;
use winit::event_loop::EventLoop;

mod app;
mod backend;
mod cli;
mod config;
mod consts;
mod first_time;
mod gui;
mod layout;
mod loaders;
mod raw_image;
mod style;
mod undo;
mod updater;

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

    if let Some(subcmd) = opts.subcmd {
        match subcmd {
            SubCommand::Contextual { subcmd } => match subcmd {
                cli::ContextualSubCommand::Install => {
                    contextual::install()?;
                }
                cli::ContextualSubCommand::Remove => {
                    contextual::remove()?;
                }
            },
            SubCommand::Init { path } => {
                let root = path.unwrap_or(std::env::current_dir()?);

                config.default_dir = Some(root.clone());

                config.save().unwrap();

                let backend = DbBackend::init_at_path(root).await?;
                backend.save().await?;
            }
            SubCommand::ResetConfig => {
                config = Config::default();
                config.save()?;
            }
            SubCommand::Gui => {
                run_gui(config).await?;
            }
            SubCommand::Update => {
                tokio::task::spawn_blocking(updater::update_app).await??;
            }
        }
    } else {
        run_gui(config).await?;
    }

    Ok(())
}

async fn run_gui(mut config: Config) -> anyhow::Result<()> {
    let root = match config.default_dir {
        Some(root) => root,
        None => {
            let root = ask_for_startup_folder().await?;

            config.default_dir = Some(root.clone());
            config.save().unwrap();
            root
        }
    };

    let (outgoing_files, app) = create_app(root).await?;

    let event_loop = EventLoop::new();
    let gui = GuiContext::create(&event_loop).await?;
    run_event_loop(event_loop, gui, outgoing_files, app);
    Ok(())
}

async fn ask_for_startup_folder() -> anyhow::Result<PathBuf> {
    loop {
        if !AsyncMessageDialog::new()
            .set_title("First Time")
            .set_description(
                "\nSince this is your first time running ArtOrganize, \
                you'll need to setup an initial database to work on!\
                \n\nIf you already have an existing database, you can select \
                that directory instead.",
            )
            .set_level(MessageLevel::Info)
            .set_buttons(MessageButtons::OkCancle)
            .show()
            .await
        {
            bail!("User refused to select a initial file.");
        }
        if let Some(item) = AsyncFileDialog::new().pick_folder().await {
            break Ok(item.path().to_path_buf());
        }
    }
}

async fn create_app(root: PathBuf) -> anyhow::Result<(mpsc::Sender<std::path::PathBuf>, App)> {
    let (outgoing_images, rx) = mpsc::channel();
    let (outgoing_files, incoming_files) = mpsc::channel();
    let db = Arc::new(RwLock::new({
        match DbBackend::from_path(root.clone()).await {
            Ok(value) => value,
            Err(_) => {
                let backend = DbBackend::init_at_path(root.clone()).await?;
                backend.save().await?;
                backend
            }
        }
    }));
    let gui_state = Arc::new(RwLock::new(GuiState::default()));
    let db_handle = start_db_task(db);
    let gui_handle = start_gui_task(
        db_handle,
        gui_state.clone(),
        outgoing_images,
        incoming_files,
    );
    let app = App {
        incoming_images: rx,
        gui_handle,
        gui_state,
    };
    Ok((outgoing_files, app))
}

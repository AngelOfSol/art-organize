// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    path::PathBuf,
    sync::{mpsc, Arc, RwLock},
    time::Duration,
};

use anyhow::bail;
use backend::{actor::start_db_task, DbBackend};
use clap::Clap;
use cli::SubCommand;
use config::Config;
use rfd::{AsyncFileDialog, AsyncMessageDialog, MessageButtons, MessageLevel};
use tokio::runtime::Builder;
use winit::event_loop::EventLoop;

mod backend;
mod cli;
mod config;
mod consts;
mod egui_app;
mod frontend;
mod layout;
mod loaders;
mod undo;
mod updater;

fn main() -> anyhow::Result<()> {
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .max_blocking_threads(12)
        .thread_keep_alive(Duration::from_secs(1))
        .enable_all()
        .build()?;

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

                let backend = DbBackend::init_at_directory(root).await?;
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

    egui_app::main().await;
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

async fn create_app(root: PathBuf) -> anyhow::Result<(mpsc::Sender<std::path::PathBuf>, ())> {
    let (outgoing_files, incoming_files) = mpsc::channel();
    let db = Arc::new(RwLock::new({
        match DbBackend::from_directory(root.clone()).await {
            Ok(value) => value,
            Err(_) => {
                let backend = DbBackend::init_at_directory(root.clone()).await?;
                backend.save().await?;
                backend
            }
        }
    }));
    let db_handle = start_db_task(db);

    Ok((outgoing_files, ()))
}

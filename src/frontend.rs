use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::{
    backend::DbBackend,
    config::Config,
    frontend::texture_storage::{ImageData, ImageStatus},
    ui_memory::MemoryExt,
    views::{gallery::Gallery, View, ViewResponse},
};
use db::BlobId;
use egui::{CentralPanel, Layout, TopBottomPanel};

pub mod blob;
pub mod category;
pub mod easy_mark_editor;
pub mod piece;
pub mod tag;
pub mod tag_editor;
pub mod texture_storage;

pub struct Frontend {
    history: Vec<Box<dyn View>>,
    image_data: ImageData,
    new_db: Arc<Mutex<Option<DbBackend>>>,
    last_save: Option<Instant>,
}

impl Frontend {
    pub fn new(image_data: ImageData) -> Self {
        Self {
            history: vec![Box::new(Gallery)],
            image_data,
            new_db: Arc::new(Mutex::new(None)),
            last_save: None,
        }
    }

    pub fn image_for(&mut self, blob_id: BlobId, db: &DbBackend) -> ImageStatus {
        self.image_data.image_for(blob_id, db)
    }
    pub fn thumbnail_for(&mut self, blob_id: BlobId, db: &DbBackend) -> ImageStatus {
        self.image_data.thumbnail_for(blob_id, db)
    }

    pub fn image_data_mut(&mut self) -> &mut ImageData {
        &mut self.image_data
    }
}

impl Frontend {
    pub fn update(&mut self, db: &mut DbBackend, ctx: &egui::CtxRef, quit: &mut bool) {
        if let Ok(Some(new_db)) = self.new_db.try_lock().map(|mut inner| inner.take()) {
            *db = new_db;
        }

        TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save").clicked() {
                        db.save().unwrap();
                        self.last_save = Some(Instant::now());
                        ui.close_menu();
                    }
                    if ui.button("Load").clicked() {
                        let handle = self.new_db.clone();
                        tokio::spawn(async move {
                            let mut path = rfd::AsyncFileDialog::new()
                                .add_filter("ArtOrganize Database", &["aodb"])
                                .pick_file()
                                .await?
                                .path()
                                .to_path_buf();

                            let db = DbBackend::from_file(path.clone()).await.ok()?;
                            let mut config = Config::load().unwrap();
                            path.pop();
                            config.default_dir = Some(path);
                            config.save().unwrap();
                            *handle.lock().unwrap() = Some(db);
                            Some(())
                        });
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        *quit = true;
                        ui.close_menu()
                    }
                });
                ui.separator();
                let mut pop_to = None;
                for (idx, view) in self.history.iter().enumerate() {
                    let last = idx == self.history.len() - 1;
                    let response = ui.selectable_label(false, &view.name(db));
                    if response.clicked() {
                        pop_to = Some(idx);
                    }

                    if !last {
                        ui.label(">");
                    }
                }
                if let Some(pop_to) = pop_to {
                    self.history.resize_with(pop_to + 1, || unreachable!());
                }

                ui.with_layout(Layout::right_to_left(), |ui| {
                    if let Some(time) = self.last_save {
                        let difference = Instant::now() - time;
                        let minutes = difference.as_secs() / 60;
                        ui.label(match minutes {
                            0 => "Saved less than 1 minute ago.".into(),
                            1 => "Saved 1 minute ago.".into(),
                            x => format!("Saved {} minutes ago.", x),
                        });
                    } else {
                        ui.label("Not saved recently.");
                    }
                });
            });
        });

        let mut current_view = self.history.pop().unwrap();

        let mut view_response = ViewResponse::Unchanged;

        current_view.side_panels(ctx, self, db);

        CentralPanel::default().show(ctx, |ui| {
            current_view.center_panel(ui, self, db);
            view_response = ui.view_response();
            ui.reset_view_response();
        });

        if let Some(open_url) = ctx.output().open_url.take() {
            let _ = open::that(&open_url.url);
        }

        self.handle_view_response(view_response, current_view);
    }

    fn handle_view_response(&mut self, view_response: ViewResponse, current_view: Box<dyn View>) {
        match view_response {
            ViewResponse::Push(new_view) => {
                self.history.push(current_view);
                self.history.push(new_view);
            }
            ViewResponse::Replace(new_view) => {
                self.history.push(new_view);
            }
            ViewResponse::Pop => {}
            ViewResponse::Unchanged => {
                self.history.push(current_view);
            }
        }
    }
}

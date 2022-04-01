use crate::{
    backend::DbBackend,
    frontend::texture_storage::{ImageData, ImageStatus},
    ui_memory::MemoryExt,
    views::{gallery::Gallery, View, ViewResponse},
};
use db::BlobId;
use egui::{CentralPanel, TopBottomPanel};

pub mod blob;
pub mod easy_mark_editor;
pub mod piece;
pub mod tag;
pub mod tag_editor;
pub mod texture_storage;

pub struct Frontend {
    history: Vec<Box<dyn View>>,
    image_data: ImageData,
}

impl Frontend {
    pub fn new(image_data: ImageData) -> Self {
        Self {
            history: vec![Box::new(Gallery)],
            image_data,
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
        TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save").clicked() {
                        db.save().unwrap();
                        ui.close_menu();
                    }
                    if ui.button("Load").clicked() {
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
                    let response = ui.selectable_label(false, &view.name());
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

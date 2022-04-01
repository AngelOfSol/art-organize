use egui::{ImageButton, PointerButton, ScrollArea};
use itertools::Itertools;

use crate::{
    backend::DbBackend,
    frontend::{texture_storage::ImageStatus, Frontend},
    ui_memory::MemoryExt,
    views::{edit_piece::EditPiece, view_piece::ViewPiece, View},
};

#[derive(Clone, Copy)]
pub struct Gallery;

impl View for Gallery {
    fn center_panel(&mut self, ui: &mut egui::Ui, frontend: &mut Frontend, db: &mut DbBackend) {
        ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (piece_id, _) in db.pieces().sorted_by_key(|(_, item)| item.added).rev() {
                        let blob_id = db
                            .blobs_for_piece(piece_id)
                            .sorted_by_key(|item| (db[item].blob_type, db[item].added))
                            .next();
                        if let Some(blob_id) = blob_id {
                            if let ImageStatus::Available(image) =
                                frontend.thumbnail_for(blob_id, db)
                            {
                                let response =
                                    ui.add(ImageButton::new(image.id, image.with_height(256.0)));
                                if response.clicked_by(PointerButton::Primary) {
                                    ui.push_view(ViewPiece {
                                        piece_id,
                                        previewed: Some(blob_id),
                                    });
                                }

                                response.context_menu(|ui| {
                                    if ui.button("Edit").clicked() {
                                        ui.push_view(EditPiece { piece_id });
                                        ui.close_menu();
                                    }
                                    if ui.button("View").clicked() {
                                        ui.push_view(ViewPiece {
                                            piece_id,
                                            previewed: Some(blob_id),
                                        });
                                        ui.close_menu();
                                    }
                                });
                            }
                        }
                    }
                });
            });
    }
    fn name(&self) -> String {
        "Gallery".into()
    }
    fn boxed_clone(&self) -> Box<dyn View> {
        Box::new(*self)
    }
}

use egui::{ImageButton, PointerButton, ScrollArea};
use itertools::Itertools;

use crate::{
    backend::DbBackend,
    frontend::{texture_storage::ImageStatus, Frontend},
    views::{edit_piece::EditPiece, view_piece::ViewPiece, View, ViewResponse},
};

pub struct Gallery;

impl View for Gallery {
    fn center_panel(
        &mut self,
        ui: &mut egui::Ui,
        frontend: &mut Frontend,
        db: &mut DbBackend,
        view_response: &mut ViewResponse,
    ) {
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
                                    view_response.push(ViewPiece {
                                        piece_id,
                                        previewed: Some(blob_id),
                                    });
                                }

                                response.context_menu(|ui| {
                                    if ui.button("Edit").clicked() {
                                        view_response.push(EditPiece { piece_id });
                                        ui.close_menu();
                                    }
                                    if ui.button("View").clicked() {
                                        view_response.push(ViewPiece {
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
}

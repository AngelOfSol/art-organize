use db::{BlobId, PieceId};
use egui::{ImageButton, ScrollArea, SidePanel, TopBottomPanel};
use itertools::Itertools;

use crate::{
    backend::DbBackend,
    frontend::{blob, piece, texture_storage::ImageStatus, Frontend},
    ui_memory::MemoryExt,
    views::{view_blob::ViewBlob, View},
};

#[derive(Clone, Copy)]
pub struct ViewPiece {
    pub piece_id: PieceId,
    pub previewed: Option<BlobId>,
}

impl View for ViewPiece {
    fn center_panel(&mut self, ui: &mut egui::Ui, frontend: &mut Frontend, db: &mut DbBackend) {
        if let Some(blob_id) = self.previewed {
            if let ImageStatus::Available(texture) = frontend.image_for(blob_id, db) {
                ui.centered_and_justified(|ui| {
                    if ui
                        .add(
                            ImageButton::new(
                                texture.id,
                                texture.scaled(ui.available_size().into()),
                            )
                            .selected(false)
                            .frame(false),
                        )
                        .double_clicked()
                    {
                        ui.push_view(ViewBlob { blob_id });
                    }
                });
            }
        } else {
            ui.label("No Image");
        }
    }

    fn side_panels(&mut self, ctx: &egui::CtxRef, frontend: &mut Frontend, db: &mut DbBackend) {
        SidePanel::left("information")
            .resizable(false)
            .show(ctx, |ui| {
                piece::info_panel(db, self.piece_id, ui);
            });
        TopBottomPanel::bottom("image_list")
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_min_height(276.0);
                ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for blob_id in db
                            .blobs_for_piece(self.piece_id)
                            .sorted_by_key(|item| (db[item].blob_type, db[item].added))
                        {
                            blob::thumbnail(ui, frontend, db, blob_id);
                        }
                    });
                });
            });
    }
    fn name(&self) -> String {
        "Piece".into()
    }
    fn boxed_clone(&self) -> Box<dyn View> {
        Box::new(*self)
    }
}

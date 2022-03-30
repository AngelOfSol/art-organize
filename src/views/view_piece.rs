use db::{BlobId, PieceId};
use egui::{Button, CtxRef, ImageButton, ScrollArea, SidePanel, TopBottomPanel};
use itertools::Itertools;

use crate::{
    backend::DbBackend,
    frontend::{piece, texture_storage::ImageStatus, Frontend},
    views::{view_blob::ViewBlob, View, ViewResponse},
};

pub struct ViewPiece {
    pub piece_id: PieceId,
    pub previewed: Option<BlobId>,
}

impl View for ViewPiece {
    fn center_panel(
        &mut self,
        ui: &mut egui::Ui,
        frontend: &mut Frontend,
        db: &mut DbBackend,
        view_response: &mut ViewResponse,
    ) {
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
                        view_response.push(ViewBlob { blob_id });
                    }
                });
            }
        } else {
            ui.label("No Image");
        }
    }

    fn side_panels(
        &mut self,
        ctx: &CtxRef,
        frontend: &mut Frontend,
        db: &mut DbBackend,
        _: &mut ViewResponse,
    ) {
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
                            match frontend.thumbnail_for(blob_id, db) {
                                ImageStatus::Available(texture) => {
                                    let response = ui.add(
                                        ImageButton::new(texture.id, texture.with_height(256.0))
                                            .selected(self.previewed == Some(blob_id)),
                                    );

                                    if response.clicked() {
                                        self.previewed = Some(blob_id);
                                    }
                                }
                                ImageStatus::Unavailable => {
                                    ui.add_sized(
                                        [256.0, 256.0],
                                        Button::new(&db[blob_id].file_name),
                                    );
                                }
                            }
                        }
                    });
                });
            });
    }
    fn name(&self) -> String {
        "View Piece".into()
    }
}

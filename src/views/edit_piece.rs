use db::PieceId;
use egui::{Button, ImageButton, ScrollArea, TopBottomPanel};
use itertools::Itertools;

use crate::{
    backend::DbBackend,
    frontend::{
        easy_mark_editor::easy_mark_editor, piece, tag_editor::tag_editor,
        texture_storage::ImageStatus, Frontend,
    },
    ui_memory::TextItemEdit,
    views::{view_blob::ViewBlob, View, ViewResponse},
};

pub struct EditPiece {
    pub piece_id: PieceId,
}

impl View for EditPiece {
    fn center_panel(
        &mut self,
        ui: &mut egui::Ui,
        _: &mut Frontend,
        db: &mut DbBackend,
        _: &mut ViewResponse,
    ) {
        ui.columns(2, |ui| {
            ui[0].vertical(|ui| {
                let piece = db.pieces.get_mut(self.piece_id).unwrap();

                ui.add(
                    TextItemEdit::new(
                        ui.make_persistent_id("external_id").with(self.piece_id),
                        &mut piece.external_id,
                    )
                    .hint_text("External ID"),
                );

                ui.add(
                    TextItemEdit::new(
                        ui.make_persistent_id("added").with(self.piece_id),
                        &mut piece.added,
                    )
                    .hint_text("Added On"),
                );

                ui.add(
                    TextItemEdit::new(
                        ui.make_persistent_id("base_price").with(self.piece_id),
                        &mut piece.base_price,
                    )
                    .hint_text("Price"),
                );
                ui.add(
                    TextItemEdit::new(
                        ui.make_persistent_id("tip_price").with(self.piece_id),
                        &mut piece.tip_price,
                    )
                    .hint_text("Tip"),
                );

                ui.separator();
                easy_mark_editor(ui, &mut piece.description);
                ui.separator();
                tag_editor(ui, self.piece_id, self.piece_id, db);
            });
            ui[1].vertical(|ui| {
                piece::info_panel(db, self.piece_id, ui);
            });
        });
    }

    fn side_panels(
        &mut self,
        ctx: &egui::CtxRef,
        frontend: &mut Frontend,
        db: &mut DbBackend,
        view_response: &mut ViewResponse,
    ) {
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
                                    let response = ui.add(ImageButton::new(
                                        texture.id,
                                        texture.with_height(256.0),
                                    ));

                                    if response.clicked() {
                                        view_response.push(ViewBlob { blob_id });
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
        "Edit Piece".into()
    }
}

use db::PieceId;
use egui::{ScrollArea, TopBottomPanel};
use itertools::Itertools;

use crate::{
    backend::DbBackend,
    frontend::{blob, easy_mark_editor::easy_mark_editor, piece, tag_editor::tag_editor, Frontend},
    ui_memory::TextItemEdit,
    views::View,
};

#[derive(Clone, Copy)]
pub struct EditPiece {
    pub piece_id: PieceId,
}

impl View for EditPiece {
    fn center_panel(&mut self, ui: &mut egui::Ui, _: &mut Frontend, db: &mut DbBackend) {
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

    fn side_panels(&mut self, ctx: &egui::CtxRef, frontend: &mut Frontend, db: &mut DbBackend) {
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
        "Edit Piece".into()
    }

    fn boxed_clone(&self) -> Box<dyn View> {
        Box::new(*self)
    }
}

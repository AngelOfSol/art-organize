use db::{BlobId, PieceId};
use egui::{ScrollArea, SidePanel, TopBottomPanel};
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
    pub previewed: Option<BlobId>,
}

impl View for EditPiece {
    fn center_panel(&mut self, ui: &mut egui::Ui, frontend: &mut Frontend, db: &mut DbBackend) {
        if let Some(blob_id) = self.previewed {
            blob::display(ui, frontend, db, blob_id);
        } else {
            ui.label("No Image");
        }
    }

    fn side_panels(&mut self, ctx: &egui::CtxRef, frontend: &mut Frontend, db: &mut DbBackend) {
        SidePanel::left("editor").resizable(false).show(ctx, |ui| {
            ui.columns(2, |ui| {
                ui[0].vertical(|ui| {
                    let piece = db.pieces.get_mut(self.piece_id).unwrap();

                    let parent_id = ui.make_persistent_id(self.piece_id);
                    ui.add(
                        TextItemEdit::new(parent_id.with("external_id"), &mut piece.external_id)
                            .hint_text("External ID"),
                    );

                    ui.add(
                        TextItemEdit::new(parent_id.with("added"), &mut piece.added)
                            .hint_text("Added On"),
                    );

                    ui.add(
                        TextItemEdit::new(parent_id.with("base_price"), &mut piece.base_price)
                            .hint_text("Price"),
                    );

                    ui.add(
                        TextItemEdit::new(parent_id.with("tip_price"), &mut piece.tip_price)
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
                            if blob::thumbnail(ui, frontend, db, blob_id).clicked() {
                                self.previewed = Some(blob_id);
                            }
                        }
                    });
                });
            });
    }

    fn name(&self, db: &DbBackend) -> String {
        format!("Edit {}...", &db[self.piece_id].description[..10])
    }

    fn boxed_clone(&self) -> Box<dyn View> {
        Box::new(*self)
    }
}

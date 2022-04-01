use crate::{
    backend::DbBackend,
    frontend::{piece, Frontend},
    views::View,
};
use egui::ScrollArea;
use itertools::Itertools;

#[derive(Clone, Copy)]
pub struct Gallery;

impl View for Gallery {
    fn center_panel(&mut self, ui: &mut egui::Ui, frontend: &mut Frontend, db: &mut DbBackend) {
        ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (piece_id, _) in db.pieces().sorted_by_key(|(_, item)| item.added).rev() {
                        piece::thumbnail(db, piece_id, frontend, ui);
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

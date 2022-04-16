use crate::{
    backend::DbBackend,
    frontend::{piece, tag, Frontend},
    views::View,
};
use egui::{ScrollArea, SidePanel};
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

    fn side_panels(&mut self, ctx: &egui::CtxRef, _: &mut Frontend, db: &mut DbBackend) {
        SidePanel::left("information").show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    tag::list(db, db.tags.keys(), ui);
                });
        });
    }
    fn name(&self, _: &DbBackend) -> String {
        "Gallery".into()
    }
    fn boxed_clone(&self) -> Box<dyn View> {
        Box::new(*self)
    }
}

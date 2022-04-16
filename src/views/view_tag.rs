use crate::{
    backend::DbBackend,
    frontend::{piece, Frontend},
    ui_memory::color32_from,
    views::View,
};
use db::TagId;
use egui::{ScrollArea, SidePanel};
use egui_demo_lib::easy_mark::easy_mark;
use itertools::Itertools;

#[derive(Clone, Copy)]
pub struct ViewTag {
    pub tag_id: TagId,
}

impl View for ViewTag {
    fn center_panel(&mut self, ui: &mut egui::Ui, frontend: &mut Frontend, db: &mut DbBackend) {
        ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for piece_id in db
                        .pieces_for_tag(self.tag_id)
                        .sorted_by_key(|piece_id| db[piece_id].added)
                        .rev()
                    {
                        piece::thumbnail(db, piece_id, frontend, ui);
                    }
                });
            });
    }
    fn side_panels(&mut self, ctx: &egui::CtxRef, _: &mut Frontend, db: &mut DbBackend) {
        SidePanel::left("information")
            .resizable(false)
            .show(ctx, |ui| {
                let tag = &db[self.tag_id];

                if let Some(category_id) = db.category_for_tag(self.tag_id) {
                    ui.colored_label(color32_from(db[category_id].color), &db[category_id].name);
                }
                ui.label(format!("Added: {}", tag.added));

                if tag.description.trim() != "" {
                    ui.separator();
                    easy_mark(ui, &tag.description);
                }
            });
    }

    fn name(&self, db: &DbBackend) -> String {
        format!("#{}", db[self.tag_id].name)
    }
    fn boxed_clone(&self) -> Box<dyn View> {
        Box::new(*self)
    }
}

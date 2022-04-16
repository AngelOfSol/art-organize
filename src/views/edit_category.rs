use crate::{
    backend::DbBackend,
    frontend::{easy_mark_editor::easy_mark_editor, Frontend},
    ui_memory::{MemoryExt, TextItemEdit},
    views::{edit_tag::EditTag, view_tag::ViewTag, View},
};
use db::CategoryId;
use egui::{ScrollArea, SidePanel};
use itertools::Itertools;

#[derive(Clone, Copy)]
pub struct EditCategory {
    pub category_id: CategoryId,
}

impl View for EditCategory {
    fn center_panel(&mut self, ui: &mut egui::Ui, _: &mut Frontend, db: &mut DbBackend) {
        ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let columns = 3;
                    ui.columns(columns, |uis| {
                        let tags = db
                            .tags_for_category(self.category_id)
                            .sorted_by_key(|tag_id| &db[tag_id].name)
                            .collect::<Vec<_>>();

                        for row in tags.chunks(columns) {
                            for (tag_id, column) in row.iter().copied().zip(0..columns) {
                                uis[column]
                                    .selectable_label(false, &db[tag_id].name)
                                    .context_menu(|ui| {
                                        if ui.button("Remove from Category").clicked() {
                                            db.tag_category.remove(&tag_id);
                                            ui.close_menu();
                                        }
                                        ui.separator();
                                        if ui.button("Edit").clicked() {
                                            ui.push_view(EditTag { tag_id });
                                            ui.close_menu();
                                        }
                                        if ui.button("View").clicked() {
                                            ui.push_view(ViewTag { tag_id });
                                            ui.close_menu();
                                        }
                                    });
                            }
                        }
                    })
                });
            });
    }
    fn side_panels(&mut self, ctx: &egui::CtxRef, _: &mut Frontend, db: &mut DbBackend) {
        SidePanel::left("information").show(ctx, |ui| {
            let category = db.categories.get_mut(self.category_id).unwrap();
            let parent_id = ui.make_persistent_id(self.category_id);
            ui.add(TextItemEdit::new(parent_id.with("Name"), &mut category.name).hint_text("Name"));
            ui.add(
                TextItemEdit::new(parent_id.with("added"), &mut category.added)
                    .hint_text("Added On"),
            );
            ui.color_edit_button_srgba_unmultiplied(&mut category.color);

            ui.separator();
            easy_mark_editor(ui, &mut category.description);
        });
    }

    fn name(&self, db: &DbBackend) -> String {
        format!("#{}", db[self.category_id].name)
    }
    fn boxed_clone(&self) -> Box<dyn View> {
        Box::new(*self)
    }
}

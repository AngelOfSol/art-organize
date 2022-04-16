use crate::{
    backend::DbBackend,
    frontend::{easy_mark_editor::easy_mark_editor, piece, Frontend},
    ui_memory::TextItemEdit,
    views::View,
};
use db::TagId;
use egui::{ComboBox, ScrollArea, SidePanel};
use itertools::Itertools;

#[derive(Clone, Copy)]
pub struct EditTag {
    pub tag_id: TagId,
}

impl View for EditTag {
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
        SidePanel::left("information").show(ctx, |ui| {
            let mut category_for_tag = db.category_for_tag(self.tag_id);
            ComboBox::from_label("Category")
                .selected_text(
                    category_for_tag
                        .as_ref()
                        .map(|category_id| db[category_id].name.clone())
                        .unwrap_or_else(|| "<none>".to_string()),
                )
                .show_ui(ui, |ui| {
                    let mut response = ui.selectable_value(&mut category_for_tag, None, "<none>");
                    for category_id in db.categories.keys() {
                        response = ui
                            .selectable_value(
                                &mut category_for_tag,
                                Some(category_id),
                                &db[category_id].name,
                            )
                            .union(response);
                    }
                    if response.changed() {
                        if let Some(category_id) = category_for_tag {
                            db.tag_category.insert(self.tag_id, category_id);
                        } else {
                            db.tag_category.remove(&self.tag_id);
                        }
                    }
                });

            let tag = db.tags.get_mut(self.tag_id).unwrap();
            let parent_id = ui.make_persistent_id(self.tag_id);

            ui.add(TextItemEdit::new(parent_id.with("name"), &mut tag.name).hint_text("Name"));
            ui.add(
                TextItemEdit::new(parent_id.with("added"), &mut tag.added).hint_text("Added On"),
            );

            ui.separator();
            easy_mark_editor(ui, &mut tag.description);
        });
    }

    fn name(&self, db: &DbBackend) -> String {
        format!("#{}", db[self.tag_id].name)
    }
    fn boxed_clone(&self) -> Box<dyn View> {
        Box::new(*self)
    }
}

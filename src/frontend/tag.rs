use db::TagId;
use egui::{Response, RichText};
use egui_demo_lib::easy_mark::easy_mark;
use itertools::Itertools;

use crate::{
    backend::DbBackend,
    frontend::category,
    ui_memory::{color32_from, MemoryExt},
    views::{edit_tag::EditTag, view_tag::ViewTag},
};

pub fn label(ui: &mut egui::Ui, db: &DbBackend, tag_id: TagId) -> Response {
    let mut text = RichText::new(&db[tag_id].name);

    if let Some(category_id) = db.category_for_tag(tag_id) {
        text = text.color(color32_from(db[category_id].color));
    }

    let response = ui.selectable_label(false, text);

    let description = &db[tag_id].description;
    if description.trim() != "" {
        response.clone().on_hover_ui(|ui| {
            easy_mark(ui, description);
        });
    }

    response.clone().context_menu(|ui| {
        if ui.button("Edit").clicked() {
            ui.push_view(EditTag { tag_id });
            ui.close_menu();
        }
        if ui.button("View").clicked() {
            ui.push_view(ViewTag { tag_id });
            ui.close_menu();
        }
    });

    response
}

pub fn list(db: &DbBackend, iter: impl Iterator<Item = TagId>, ui: &mut egui::Ui) {
    let iter = iter.collect::<Vec<_>>().into_iter();
    for category_id in iter
        .clone()
        .flat_map(|tag| db.category_for_tag(tag))
        .sorted_by_key(|category_id| &db[category_id].name)
        .dedup()
    {
        category::label(ui, db, category_id);

        ui.indent("category_indent", |ui| {
            for tag_id in iter
                .clone()
                .filter(|tag_id| db.category_for_tag(*tag_id) == Some(category_id))
                .sorted_by_key(|tag_id| &db[tag_id].name)
            {
                label(ui, db, tag_id);
            }
        });
    }
    for tag_id in iter
        .clone()
        .filter(|tag_id| db.category_for_tag(*tag_id).is_none())
        .sorted_by_key(|tag_id| &db[tag_id].name)
    {
        label(ui, db, tag_id);
    }
}

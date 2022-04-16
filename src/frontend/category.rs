use db::CategoryId;
use egui::{Response, RichText, Ui};
use egui_demo_lib::easy_mark::easy_mark;

use crate::{
    backend::DbBackend,
    ui_memory::{color32_from, MemoryExt},
    views::edit_category::EditCategory,
};

pub fn label(ui: &mut Ui, db: &DbBackend, category_id: CategoryId) -> Response {
    let response = ui
        .selectable_label(
            false,
            RichText::new(&db[category_id].name).color(color32_from(db[category_id].color)),
        )
        .context_menu(|ui| {
            if ui.button("Edit").clicked() {
                ui.push_view(EditCategory { category_id });
                ui.close_menu();
            }
        });

    if !db[category_id].description.trim().is_empty() {
        response.on_hover_ui(|ui| {
            easy_mark(ui, &db[category_id].description);
        })
    } else {
        response
    }
}

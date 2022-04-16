use db::CategoryId;
use egui::{Response, Ui};

use crate::backend::DbBackend;

pub fn label(ui: &mut Ui, db: &DbBackend, category_id: CategoryId) -> Response {
    ui.selectable_label(false, &db[category_id].name)
        .context_menu(|ui| {
            if ui.button("Edit").clicked() {
                // ui.push_view(EditTag { tag_id });
                ui.close_menu();
            }
        })
}

use db::TagId;
use egui::{Color32, Response, RichText};

use crate::backend::DbBackend;

pub fn label(ui: &mut egui::Ui, db: &DbBackend, tag_id: TagId) -> Response {
    let mut text = RichText::new(&db[tag_id].name);

    if let Some(category_id) = db.category_for_tag(tag_id) {
        text = text.color(Color32::from_rgb(
            db[category_id].color[0],
            db[category_id].color[1],
            db[category_id].color[2],
        ));
    }

    let response = ui.selectable_label(false, text);

    let description = &db[tag_id].description;
    if description.trim() != "" {
        response.clone().on_hover_ui(|ui| {
            ui.label(description);
        });
    }

    response
}

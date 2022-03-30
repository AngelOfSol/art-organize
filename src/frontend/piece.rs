use db::PieceId;
use egui_demo_lib::easy_mark::easy_mark;
use itertools::Itertools;

use crate::{backend::DbBackend, frontend::tag};

pub fn info_panel(db: &mut DbBackend, piece_id: PieceId, ui: &mut egui::Ui) {
    let piece = &db[piece_id];
    ui.label(format!(
        "External ID: {}",
        piece.external_id.as_deref().unwrap_or("<none>")
    ));
    ui.label(format!("Added: {}", piece.added));
    if let Some(price) = piece.base_price {
        ui.label(format!("Price: ${}", price));
    }
    if let Some(price) = piece.tip_price {
        ui.label(format!("Tip: ${}", price));
    }
    if piece.description.trim() != "" {
        ui.separator();
        easy_mark(ui, &piece.description);
    }
    ui.separator();
    for category_id in db
        .tags_for_piece(piece_id)
        .flat_map(|tag| db.category_for_tag(tag))
        .sorted_by_key(|category_id| &db[category_id].name)
        .dedup()
    {
        ui.label(&db[category_id].name);

        ui.indent("category_indent", |ui| {
            for tag_id in db
                .tags_for_piece(piece_id)
                .filter(|tag_id| db.category_for_tag(*tag_id) == Some(category_id))
                .sorted_by_key(|tag_id| &db[tag_id].name)
            {
                tag::label(ui, db, tag_id);
            }
        });
    }
    for tag_id in db
        .tags_for_piece(piece_id)
        .filter(|tag_id| db.category_for_tag(*tag_id).is_none())
        .sorted_by_key(|tag_id| &db[tag_id].name)
    {
        tag::label(ui, db, tag_id);
    }
}

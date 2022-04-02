use db::PieceId;
use egui::{ImageButton, PointerButton};
use egui_demo_lib::easy_mark::easy_mark;
use itertools::Itertools;

use crate::{
    backend::DbBackend,
    frontend::{
        tag::{self},
        texture_storage::ImageStatus,
        Frontend,
    },
    ui_memory::MemoryExt,
    views::{edit_piece::EditPiece, view_piece::ViewPiece},
};
pub fn thumbnail(db: &DbBackend, piece_id: PieceId, frontend: &mut Frontend, ui: &mut egui::Ui) {
    let blob_id = db
        .blobs_for_piece(piece_id)
        .sorted_by_key(|item| (db[item].blob_type, db[item].added))
        .next();
    if let Some(blob_id) = blob_id {
        if let ImageStatus::Available(image) = frontend.thumbnail_for(blob_id, db) {
            let response = ui.add(ImageButton::new(image.id, image.with_height(256.0)));
            if response.clicked_by(PointerButton::Primary) {
                ui.push_view(ViewPiece {
                    piece_id,
                    previewed: Some(blob_id),
                });
            }

            response.context_menu(|ui| {
                if ui.button("Edit").clicked() {
                    ui.push_view(EditPiece {
                        piece_id,
                        previewed: Some(blob_id),
                    });
                    ui.close_menu();
                }
                if ui.button("View").clicked() {
                    ui.push_view(ViewPiece {
                        piece_id,
                        previewed: Some(blob_id),
                    });
                    ui.close_menu();
                }
            });
        }
    }
}

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
    tag::list(db, db.tags_for_piece(piece_id), ui);
}

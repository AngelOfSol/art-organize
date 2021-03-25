use chrono::Local;
use db::{BlobType, Db, Tag, TagCategory};

use super::{GuiHandle, GuiView};
use crate::app::widgets::*;

pub struct Gallery;

impl GuiView for Gallery {
    fn draw_main(
        &mut self,
        gui_handle: &GuiHandle,
        gui_state: &super::InnerGuiState,
        ui: &imgui::Ui<'_>,
    ) {
        let db = gui_handle.db.read().unwrap();

        let blobs = db.pieces().filter_map(|(id, _)| {
            let mut blobs = db.blobs_for_piece(id);
            blobs
                .clone()
                .find(|id| db[id].blob_type == BlobType::Canon)
                .or_else(|| blobs.find(|id| db[id].blob_type == BlobType::Variant))
        });

        if let Some(id) =
            gallery::render(ui, blobs, &gui_handle, &gui_state.thumbnails, |blob, ui| {
                let piece_id = db.pieces_for_blob(blob).next().unwrap();
                piece::view(piece_id, &db, ui);
            })
        {
            gui_handle.request_view_piece(db.pieces_for_blob(id).next().unwrap());
        }
    }

    fn draw_explorer(
        &mut self,
        gui_handle: &GuiHandle,
        gui_state: &super::InnerGuiState,
        ui: &imgui::Ui<'_>,
    ) {
        for i in 0..10u32 {
            let t = Tag {
                name: format!("tag_{}", i),
                description: format!("My test description {}", i),
                added: Local::today().naive_local(),
                links: Vec::new(),
            };
            let tg = TagCategory {
                name: format!("category_{}", i),
                color: [(i * 128 / 10 + 120) as u8, 0, 0, 255],
                added: Local::today().naive_local(),
                ..TagCategory::default()
            };

            tag::gallery(ui, &t, &tg);
        }
    }
}

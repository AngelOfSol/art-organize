use db::BlobType;
use itertools::Itertools;
use tag::ItemViewResponse;

use super::{piece::PieceView, tag::TagView, GuiHandle, GuiView};
use crate::app::widgets::*;

#[derive(Debug)]
pub struct Gallery;

impl GuiView for Gallery {
    fn update(&self, _: &super::GuiHandle) {}
    fn draw_main(
        &mut self,
        gui_handle: &GuiHandle,
        gui_state: &super::InnerGuiState,
        ui: &imgui::Ui<'_>,
    ) {
        let db = gui_handle.db.read().unwrap();

        let blobs = db
            .pieces()
            .sorted_by_key(|(_, piece)| piece.added)
            .into_iter()
            .rev()
            .filter_map(|(id, _)| {
                let mut blobs = db.blobs_for_piece(id);
                blobs
                    .clone()
                    .find(|id| db[id].blob_type == BlobType::Canon)
                    .or_else(|| blobs.find(|id| db[id].blob_type == BlobType::Variant))
            });

        if let Some(id) = gallery::render(
            ui,
            blobs,
            &gui_handle,
            &gui_state.thumbnails,
            |blob| &db[db.pieces_for_blob(blob).next().unwrap()].name,
            |blob| {
                let piece_id = db.pieces_for_blob(blob).next().unwrap();
                piece::tooltip(piece_id, &db, ui);
            },
        ) {
            gui_handle.goto(PieceView {
                id: db.pieces_for_blob(id).next().unwrap(),
                edit: false,
            });
        }
    }

    fn draw_explorer(
        &mut self,
        gui_handle: &GuiHandle,
        _: &super::InnerGuiState,
        ui: &imgui::Ui<'_>,
    ) {
        let db = gui_handle.db.read().unwrap();
        for (tag_id, _) in db
            .tags()
            .filter(|(id, _)| db.pieces_for_tag(*id).count() > 0)
            .sorted_by_key(|(id, _)| db.pieces_for_tag(*id).count())
            // tag_list is sorted ascending, and we want the greatest 20, not the least 20
            .rev()
            .take(20)
            .sorted_by_key(|(_, tag)| &tag.name)
        {
            match tag::item_view(ui, &db, tag_id) {
                ItemViewResponse::None => {}
                ItemViewResponse::Add => {}
                ItemViewResponse::AddNegated => {}
                ItemViewResponse::Open => {
                    gui_handle.goto(TagView {
                        id: tag_id,
                        edit: false,
                    });
                }
            }
        }
    }
    fn label(&self) -> &'static str {
        "Gallery"
    }
}

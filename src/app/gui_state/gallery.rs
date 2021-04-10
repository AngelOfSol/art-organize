use db::BlobType;
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

        let blobs = db.pieces().filter_map(|(id, _)| {
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
        let mut tag_list = db.tags().collect::<Vec<_>>();
        tag_list.sort_by_key(|(id, _)| db.pieces_for_tag(*id).count());
        let mut tag_list = tag_list.into_iter().take(20).collect::<Vec<_>>();
        tag_list.sort_by_key(|(_, tag)| &tag.name);
        for (tag_id, _) in tag_list {
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
}

use super::{piece::PieceView, GuiView};
use crate::app::widgets::*;
use db::TagId;
use imgui::im_str;
use itertools::Itertools;
use tag::EditTagResponse;

#[derive(Debug)]
pub struct TagView {
    pub id: TagId,
    pub edit: bool,
}

impl GuiView for TagView {
    fn update(&self, gui_handle: &super::GuiHandle) {
        let db = gui_handle.db.read().unwrap();
        if !db.exists(self.id) {
            gui_handle.go_back();
        }
    }
    fn draw_main(
        &mut self,
        gui_handle: &super::GuiHandle,
        gui_state: &super::InnerGuiState,
        ui: &imgui::Ui<'_>,
    ) {
        let db = gui_handle.db.read().unwrap();

        let blob_ids = db
            .pieces_for_tag(self.id)
            .sorted_by_key(|piece_id| &db[piece_id].added)
            .rev()
            .filter_map(|piece_id| db.primary_blob_for_piece(piece_id));

        if let Some(blob_id) = gallery::render(
            ui,
            blob_ids,
            gui_handle,
            &gui_state.thumbnails,
            |blob_id| &db[db.pieces_for_blob(blob_id).next().unwrap()].description,
            |blob_id| piece::tooltip(db.pieces_for_blob(blob_id).next().unwrap(), &db, ui),
        ) {
            let piece_id = db.pieces_for_blob(blob_id).next().unwrap();
            gui_handle.goto(PieceView {
                id: piece_id,
                edit: false,
            });
        }
    }

    fn draw_explorer(
        &mut self,
        gui_handle: &super::GuiHandle,
        _: &super::InnerGuiState,
        ui: &imgui::Ui<'_>,
    ) {
        let db = gui_handle.db.read().unwrap();
        if db.exists(self.id) {
            if ui.button(&im_str!("{}", if self.edit { "View" } else { "Edit" })) {
                self.edit = !self.edit;
            }
            if !self.edit {
                tag::view(self.id, &db, ui);
            } else {
                match tag::edit(self.id, &db, ui) {
                    EditTagResponse::None => {}
                    EditTagResponse::Edit(edit) => {
                        gui_handle.update_tag(edit);
                    }
                    EditTagResponse::Delete => {
                        gui_handle.delete_tag(self.id);
                        gui_handle.go_back();
                    }
                    EditTagResponse::AttachCategory(attach) => {
                        gui_handle.attach_category(attach);
                    }
                }
            }
        }
    }
    fn label(&self) -> &'static str {
        "Tag"
    }
}

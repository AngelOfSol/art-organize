use super::{blob::BlobView, GuiView};
use crate::app::widgets::*;
use crate::consts::*;
use db::{BlobType, PieceId};
use imgui::{im_str, ChildWindow, CollapsingHeader};
use piece::EditPieceResponse;
use strum::IntoEnumIterator;

#[derive(Debug)]
pub struct PieceView {
    pub id: PieceId,
    pub edit: bool,
}

impl GuiView for PieceView {
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

        let blob_ids = db.blobs_for_piece(self.id);

        for blob_type in BlobType::iter() {
            let _id = ui.push_id(&im_str!("{}", blob_type));
            if CollapsingHeader::new(&im_str!("{}", blob_type))
                .default_open(true)
                .build(ui)
            {
                let _group = ui.begin_group();
                let blob_ids_of_type = blob_ids
                    .clone()
                    .filter(|blob| db[blob].blob_type == blob_type);
                if let Some(id) = gallery::render(
                    ui,
                    blob_ids_of_type,
                    &gui_handle,
                    &gui_state.thumbnails,
                    |blob_id| &db[blob_id].file_name,
                    |blob_id| {
                        blob::tooltip(blob_id, &db, ui);
                    },
                ) {
                    gui_handle.goto(BlobView { id, edit: false });
                }

                if ui.content_region_avail()[0] < THUMBNAIL_SIZE + IMAGE_BUFFER {
                    ui.new_line();
                } else {
                    ui.same_line();
                }
                ChildWindow::new(im_str!("add button"))
                    .draw_background(false)
                    .size([THUMBNAIL_SIZE + IMAGE_BUFFER; 2])
                    .build(ui, || {
                        ui.set_cursor_pos([IMAGE_BUFFER / 2.0; 2]);
                        if ui.button_with_size(im_str!("+"), [THUMBNAIL_SIZE; 2]) {
                            gui_handle.ask_blobs_for_piece(self.id, blob_type);
                        };
                    });
            }

            if ui.is_mouse_hovering_rect(
                ui.item_rect_min(),
                [
                    ui.cursor_screen_pos()[0] + ui.content_region_avail()[0],
                    ui.item_rect_max()[1],
                ],
            ) {
                for file in gui_handle.incoming_files.try_iter() {
                    gui_handle.new_blob_from_file(self.id, blob_type, file);
                }
            }
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
                piece::view_with_tags(self.id, &db, ui);
            } else {
                match piece::edit(self.id, &db, ui) {
                    EditPieceResponse::None => {}
                    EditPieceResponse::Edit(edit) => {
                        gui_handle.update_piece(edit);
                    }
                    EditPieceResponse::Delete => {
                        gui_handle.delete_piece(self.id);
                        gui_handle.go_back();
                    }
                    EditPieceResponse::AttachTag(attach_tag) => {
                        gui_handle.attach_tag(attach_tag);
                    }
                    EditPieceResponse::RemoveTag(remove_tag) => {
                        gui_handle.remove_tag(remove_tag);
                    }
                }
            }
        }
    }
}

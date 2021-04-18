use super::{blob::BlobView, tag::TagView, GuiView};
use crate::app::widgets::*;
use crate::consts::*;
use db::{BlobType, PieceId};
use glam::Vec2;
use imgui::{im_str, ChildWindow, TabBar, TabItem};
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

        TabBar::new(im_str!("Piece Tab Bar")).build(ui, || {
            for blob_type in BlobType::iter() {
                let _id = ui.push_id(&im_str!("{}", blob_type));
                let blob_ids_of_type = blob_ids
                    .clone()
                    .filter(|blob| db[blob].blob_type == blob_type);
                TabItem::new(&im_str!(
                    "{0} ({1})###{0}",
                    blob_type,
                    blob_ids_of_type.clone().count()
                ))
                .build(ui, || {
                    let _group = ui.begin_group();
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

                    ChildWindow::new(im_str!("add button"))
                        .size([THUMBNAIL_SIZE + IMAGE_BUFFER; 2])
                        .draw_background(false)
                        .build(ui, || {
                            ui.set_cursor_pos(
                                (Vec2::from(ui.cursor_pos()) + Vec2::splat(IMAGE_BUFFER) / 2.0)
                                    .into(),
                            );
                            if ui.button_with_size(im_str!("+"), [THUMBNAIL_SIZE; 2]) {
                                gui_handle.ask_blobs_for_piece(self.id, blob_type);
                            }
                        });
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
                });
            }
        });
    }

    fn draw_explorer(
        &mut self,
        gui_handle: &super::GuiHandle,
        _: &super::InnerGuiState,
        ui: &imgui::Ui<'_>,
    ) {
        let db = gui_handle.db.read().unwrap();
        if db.exists(self.id) {
            if !self.edit {
                piece::view(self.id, &db, ui);

                self.edit = ui.button(im_str!("Edit"));

                ui.separator();
                if let Some((tag_id, response)) = piece::view_tags(self.id, &db, ui) {
                    match response {
                        tag::ItemViewResponse::None => unreachable!(),
                        tag::ItemViewResponse::Add => {}
                        tag::ItemViewResponse::AddNegated => {}
                        tag::ItemViewResponse::Open => {
                            gui_handle.goto(TagView {
                                id: tag_id,
                                edit: false,
                            });
                        }
                    }
                }
            } else {
                let piece_edit = piece::edit(self.id, &db, ui);

                self.edit = !ui.button(im_str!("View"));
                ui.separator();
                let tag_edit = piece::edit_tags(self.id, &db, ui);
                if let Some(action) = piece_edit.or(tag_edit) {
                    match action {
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
                        EditPieceResponse::OpenTag(tag_id) => {
                            gui_handle.goto(TagView {
                                id: tag_id,
                                edit: false,
                            });
                        }
                    }
                }
            }
        }
    }
    fn label(&self) -> &'static str {
        "Piece"
    }
}

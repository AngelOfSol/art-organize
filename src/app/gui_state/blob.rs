use super::GuiView;
use crate::app::widgets::*;
use blob::EditBlobResponse;
use db::BlobId;
use imgui::im_str;

#[derive(Debug)]
pub struct BlobView {
    pub id: BlobId,
    pub edit: bool,
}

impl GuiView for BlobView {
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
        let content_region = ui.content_region_avail();
        if let Some(image) = gui_state.images.get(&self.id) {
            let zoom = (1.0
                / (image.width as f32 / content_region[0])
                    .max(image.height as f32 / content_region[1]))
            .min(1.0);

            let size = [image.width as f32 * zoom, image.height as f32 * zoom];

            let padded = [
                0.5 * (content_region[0] - size[0]) + ui.cursor_pos()[0],
                0.5 * (content_region[1] - size[1]) + ui.cursor_pos()[1],
            ];

            ui.set_cursor_pos(padded);

            imgui::Image::new(image.data, size).build(ui);
        } else {
            gui_handle.request_load_image(self.id);
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
                blob::view(self.id, &db, ui);
            } else {
                match blob::edit(self.id, &db, ui) {
                    EditBlobResponse::None => {}
                    EditBlobResponse::Changed(data) => {
                        gui_handle.update_blob(data);
                    }
                    EditBlobResponse::Deleted(id) => {
                        gui_handle.delete_blob(id);
                        gui_handle.go_back();
                    }
                };
            }
        }
    }
}

use std::collections::BTreeMap;

use db::BlobId;
use glam::Vec2;
use imgui::{im_str, Ui};

use crate::{
    app::gui_state::GuiHandle,
    consts::{IMAGE_BUFFER, THUMBNAIL_SIZE},
    raw_image::TextureImage,
};

use super::blob;

pub fn render<I: Iterator<Item = BlobId>, F: Fn(BlobId)>(
    ui: &Ui,
    blobs: I,
    gui_handle: &GuiHandle,
    thumbnails: &BTreeMap<BlobId, TextureImage>,
    tooltip: F,
) -> Option<BlobId> {
    let mut ret = None;

    for blob in blobs {
        let label = im_str!("##{:?}", blob);
        if let Some(thumbnail) = thumbnails.get(&blob) {
            // TODO integrate loading button into it
            match blob::thumbnail_button(&label, thumbnail, ui) {
                blob::ThumbnailResponse::None => {}
                blob::ThumbnailResponse::Hovered => {
                    ui.tooltip(|| {
                        tooltip(blob);
                    });
                }
                blob::ThumbnailResponse::Clicked => {
                    ret = Some(blob);
                }
            }
        } else {
            imgui::ChildWindow::new(&label)
                .size([THUMBNAIL_SIZE + IMAGE_BUFFER; 2])
                .draw_background(false)
                .build(ui, || {
                    ui.set_cursor_pos(
                        (Vec2::from(ui.cursor_pos()) + Vec2::splat(IMAGE_BUFFER) / 2.0).into(),
                    );
                    if ui.button_with_size(im_str!("Loading..."), [THUMBNAIL_SIZE; 2]) {
                        ret = Some(blob);
                    }
                    if ui.is_item_visible() {
                        gui_handle.request_load_image(blob);
                    }
                    if ui.is_item_hovered() {
                        ui.tooltip(|| {
                            tooltip(blob);
                        });
                    }
                });
        }
        ui.same_line();
        if ui.content_region_avail()[0] < THUMBNAIL_SIZE + IMAGE_BUFFER {
            ui.new_line();
        }
    }
    ui.new_line();

    ret
}

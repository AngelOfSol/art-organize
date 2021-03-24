use std::collections::BTreeMap;

use db::BlobId;
use imgui::{im_str, Ui};

use crate::{
    consts::{IMAGE_BUFFER, THUMBNAIL_SIZE},
    raw_image::TextureImage,
};

use super::{blob, gui_state::GuiHandle};

pub fn render<I: Iterator<Item = BlobId>, F: Fn(BlobId, &Ui<'_>)>(
    ui: &Ui,
    blobs: I,
    gui_handle: &GuiHandle,
    thumbnails: &BTreeMap<BlobId, TextureImage>,
    tooltip: F,
) -> Option<BlobId> {
    let mut ret = None;

    for blob in blobs {
        if let Some(thumbnail) = thumbnails.get(&blob) {
            match blob::thumbnail_button(&im_str!("##{:?}", blob), thumbnail, ui) {
                blob::ThumbnailResponse::None => {}
                blob::ThumbnailResponse::Hovered => {
                    ui.tooltip(|| {
                        tooltip(blob, ui);
                    });
                }
                blob::ThumbnailResponse::Clicked => {
                    ret = Some(blob);
                }
            }
        } else {
            ui.dummy([THUMBNAIL_SIZE + IMAGE_BUFFER; 2]);
            if ui.is_item_visible() {
                gui_handle.request_load_image(blob);
            }
        }
        ui.same_line();
        if ui.content_region_avail()[0] < THUMBNAIL_SIZE + IMAGE_BUFFER {
            ui.new_line();
        }
    }
    ui.new_line();

    ret
}

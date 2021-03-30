use db::{commands::EditBlob, Blob, BlobId, Db};
use imgui::{im_str, ImStr, PopupModal, Ui};

use crate::{
    consts::{IMAGE_BUFFER, THUMBNAIL_SIZE},
    raw_image::TextureImage,
};

use super::{combo_box, date};

pub fn tooltip(blob_id: BlobId, db: &Db, ui: &Ui<'_>) {
    let blob = &db[blob_id];
    ui.text(&im_str!("File Name: {}", blob.file_name));
    ui.text(&im_str!("Blob Type: {}", blob.blob_type));
    ui.text(&im_str!("Hash: {:x}", blob.hash));
    date::view("Added", &blob.added, ui);
}

pub fn view(blob_id: BlobId, db: &Db, ui: &Ui<'_>) {
    let blob = &db[blob_id];
    ui.text_wrapped(&im_str!("File Name: {}", blob.file_name));
    ui.text_wrapped(&im_str!("Blob Type: {}", blob.blob_type));
    ui.text_wrapped(&im_str!("Hash: {:x}", blob.hash));
    date::view("Added", &blob.added, ui);
}

pub enum EditBlobResponse {
    None,
    Changed(EditBlob),
    Deleted(BlobId),
}

pub fn edit(blob_id: BlobId, db: &Db, ui: &Ui<'_>) -> EditBlobResponse {
    let blob = &db[blob_id];

    let mut buf = blob.file_name.clone().into();
    ui.input_text(im_str!("File Name"), &mut buf)
        .resize_buffer(true)
        .build();

    if ui.is_item_deactivated_after_edit() {
        return EditBlobResponse::Changed(EditBlob {
            id: blob_id,
            data: Blob {
                file_name: buf.to_string(),
                ..blob.clone()
            },
        });
    }

    if let Some(blob_type) = combo_box(ui, im_str!("Blob Type"), &blob.blob_type) {
        return EditBlobResponse::Changed(EditBlob {
            id: blob_id,
            data: Blob {
                blob_type,
                ..blob.clone()
            },
        });
    }

    ui.text_wrapped(&im_str!("Hash: {:x}", blob.hash));
    if let Some(data) = date::edit(im_str!("Added"), &blob.added, ui) {
        return EditBlobResponse::Changed(EditBlob {
            id: blob_id,
            data: Blob {
                added: data,
                ..blob.clone()
            },
        });
    }

    if ui.button(im_str!("Delete")) {
        ui.open_popup(im_str!("Confirm Delete"));
    }

    let mut result = EditBlobResponse::None;
    PopupModal::new(im_str!("Confirm Delete"))
        .movable(false)
        .resizable(false)
        .collapsible(false)
        .always_auto_resize(true)
        .build(ui, || {
            ui.text(im_str!("Are you sure you want to delete this?"));

            if ui.button(im_str!("Yes, delete.")) {
                result = EditBlobResponse::Deleted(blob_id);
                ui.close_current_popup();
            }
            ui.same_line();

            if ui.button(im_str!("Cancel")) {
                ui.close_current_popup();
            }
        });

    result
}

pub enum ThumbnailResponse {
    None,
    Hovered,
    Clicked,
}

pub fn thumbnail_button(label: &ImStr, thumbnail: &TextureImage, ui: &Ui<'_>) -> ThumbnailResponse {
    let mut response = ThumbnailResponse::None;
    imgui::ChildWindow::new(label)
        .size([THUMBNAIL_SIZE + IMAGE_BUFFER; 2])
        .draw_background(false)
        .build(ui, || {
            let (size, padding) = rescale(thumbnail, [THUMBNAIL_SIZE; 2]);
            ui.set_cursor_pos([
                ui.cursor_pos()[0] + padding[0] / 2.0 + IMAGE_BUFFER / 2.0,
                ui.cursor_pos()[1] + padding[1] / 2.0 + IMAGE_BUFFER / 2.0,
            ]);

            if imgui::ImageButton::new(thumbnail.data, size).build(ui) {
                response = ThumbnailResponse::Clicked
            } else if ui.is_item_hovered() {
                response = ThumbnailResponse::Hovered
            }
        });

    response
}

fn rescale(image: &TextureImage, max_size: [f32; 2]) -> ([f32; 2], [f32; 2]) {
    rescale_with_zoom(image, max_size, 1.0)
}
fn rescale_with_zoom(image: &TextureImage, max_size: [f32; 2], zoom: f32) -> ([f32; 2], [f32; 2]) {
    let size = [image.width as f32 * zoom, image.height as f32 * zoom];
    let aspect_ratio = size[0] / size[1];
    let new_aspect_ratio = max_size[0] / max_size[1];

    let size = if size[0] <= max_size[0] && size[1] <= max_size[1] {
        size
    } else {
        let use_width = aspect_ratio >= new_aspect_ratio;

        if use_width {
            [max_size[0], size[1] * max_size[0] / size[0]]
        } else {
            [size[0] * max_size[1] / size[1], max_size[1]]
        }
    };

    (size, [max_size[0] - size[0], max_size[1] - size[1]])
}

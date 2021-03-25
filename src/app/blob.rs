use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::PathBuf,
};

use chrono::Local;
use db::{Blob, BlobId, BlobType, Db};
use imgui::{im_str, ImStr, Ui};

use crate::{
    consts::{IMAGE_BUFFER, THUMBNAIL_SIZE},
    raw_image::TextureImage,
};

use super::date;

pub async fn from_path(path: PathBuf, blob_type: BlobType) -> anyhow::Result<Blob> {
    let raw_data = tokio::fs::read(&path).await?;
    let mut hash = DefaultHasher::new();
    raw_data.hash(&mut hash);
    let hash = hash.finish();

    Ok(Blob {
        file_name: path.file_name().unwrap().to_string_lossy().into_owned(),
        hash,
        blob_type,
        added: Local::today().naive_local(),
    })
}

// TODO rename tooltip view?
pub fn view(blob_id: BlobId, db: &Db, ui: &Ui<'_>) {
    let blob = &db[blob_id];
    ui.text(&im_str!("File Name: {}", blob.file_name));
    ui.text(&im_str!("Blob Type: {}", blob.blob_type));
    ui.text(&im_str!("Hash: {:x}", blob.hash));
    date::view("Added", &blob.added, ui);
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

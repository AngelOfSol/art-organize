use db::BlobId;
use egui::{Button, ImageButton, Response, Ui};

use crate::{
    backend::DbBackend,
    frontend::{texture_storage::ImageStatus, Frontend},
    ui_memory::MemoryExt,
    views::{edit_blob::EditBlob, view_blob::ViewBlob},
};

pub fn thumbnail(
    ui: &mut Ui,
    frontend: &mut Frontend,
    db: &mut DbBackend,
    blob_id: BlobId,
) -> Response {
    let response = match frontend.thumbnail_for(blob_id, db) {
        ImageStatus::Available(texture) => {
            let response = ui.add(ImageButton::new(texture.id, texture.with_height(256.0)));

            if response.double_clicked() {
                ui.push_view(ViewBlob { blob_id });
            }
            response
        }
        ImageStatus::Unavailable => {
            ui.add_sized([256.0, 256.0], Button::new(&db[blob_id].file_name))
        }
    };

    response
        .clone()
        .context_menu(|ui| context_menu(ui, blob_id));

    response
}

fn context_menu(ui: &mut Ui, blob_id: BlobId) {
    if ui.button("Edit").clicked() {
        ui.push_view(EditBlob { blob_id });
        ui.close_menu();
    }
    if ui.button("View").clicked() {
        ui.push_view(ViewBlob { blob_id });
        ui.close_menu();
    }
}

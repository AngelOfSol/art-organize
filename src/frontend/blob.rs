use db::BlobId;
use egui::{Button, ImageButton, Response, Ui};

use crate::{
    backend::DbBackend,
    frontend::{texture_storage::ImageStatus, Frontend},
    ui_memory::MemoryExt,
    views::{edit_blob::EditBlob, view_blob::ViewBlob},
};

pub fn display(ui: &mut Ui, frontend: &mut Frontend, db: &mut DbBackend, blob_id: BlobId) {
    if let ImageStatus::Available(texture) = frontend.image_for(blob_id, db) {
        ui.centered_and_justified(|ui| {
            ui.add(
                ImageButton::new(texture.id, texture.scaled(ui.available_size().into()))
                    .selected(false)
                    .frame(false),
            )
            .context_menu(|ui| {
                context_menu(ui, db, blob_id);
            });
        });
    }
}

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
            let response = ui.add_sized([256.0, 256.0], Button::new(&db[blob_id].file_name));
            if response.double_clicked() {
                ui.push_view(ViewBlob { blob_id });
            }
            response
        }
    };

    response
        .clone()
        .context_menu(|ui| context_menu(ui, db, blob_id));

    response
}

fn context_menu(ui: &mut Ui, db: &mut DbBackend, blob_id: BlobId) {
    if ui.button("Save to File").clicked() {
        let (storage, file_name) = { (db.storage_for(blob_id), db[blob_id].file_name.clone()) };

        tokio::spawn(async move {
            let mut dialog = rfd::AsyncFileDialog::new().set_file_name(&file_name);
            if let Some(ext) = std::path::Path::new(&file_name)
                .extension()
                .and_then(|inner| inner.to_str())
            {
                dialog = dialog.add_filter("Image", &[ext]);
            }
            let file = if let Some(files) = dialog.save_file().await {
                files
            } else {
                return;
            };

            tokio::fs::copy(storage, file.path()).await.unwrap();
        });

        ui.close_menu();
    }
    ui.separator();
    if ui.button("Edit").clicked() {
        ui.push_view(EditBlob { blob_id });
        ui.close_menu();
    }
    if ui.button("View").clicked() {
        ui.push_view(ViewBlob { blob_id });
        ui.close_menu();
    }
}

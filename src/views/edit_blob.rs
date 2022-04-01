use db::{BlobId, BlobType};
use egui::SidePanel;
use strum::IntoEnumIterator;

use crate::{
    backend::DbBackend,
    frontend::{texture_storage::ImageStatus, Frontend},
    ui_memory::TextItemEdit,
    views::View,
};

#[derive(Clone, Copy)]
pub struct EditBlob {
    pub blob_id: BlobId,
}

impl View for EditBlob {
    fn boxed_clone(&self) -> Box<dyn View> {
        Box::new(*self)
    }

    fn name(&self) -> String {
        "Edit Blob".into()
    }
    fn center_panel(&mut self, ui: &mut egui::Ui, frontend: &mut Frontend, db: &mut DbBackend) {
        if let ImageStatus::Available(texture) = frontend.image_for(self.blob_id, db) {
            ui.centered_and_justified(|ui| {
                ui.image(texture.id, texture.scaled(ui.available_size().into()));
            });
        }
    }

    fn side_panels(&mut self, ctx: &egui::CtxRef, _: &mut Frontend, db: &mut DbBackend) {
        SidePanel::left("left_edit_blob_panel").show(ctx, |ui| {
            let blob = db.blobs.get_mut(self.blob_id).unwrap();

            let parent_id = ui.make_persistent_id(self.blob_id);

            ui.add(
                TextItemEdit::new(parent_id.with("File Name"), &mut blob.file_name)
                    .hint_text("File Name"),
            );
            ui.add(TextItemEdit::new(parent_id.with("Added"), &mut blob.added).hint_text("Added"));
            ui.label(format!("Hash: {:#x}", blob.hash));

            ui.label("Type");
            ui.separator();
            ui.indent(parent_id.with("indent"), |ui| {
                for blob_type in BlobType::iter() {
                    ui.selectable_value(&mut blob.blob_type, blob_type, blob_type.to_string());
                }
            });
        });
    }
}

use db::BlobId;

use crate::{
    backend::DbBackend,
    frontend::{texture_storage::ImageStatus, Frontend},
    views::{View, ViewResponse},
};

pub struct ViewBlob {
    pub blob_id: BlobId,
}

impl View for ViewBlob {
    fn center_panel(
        &mut self,
        ui: &mut egui::Ui,
        frontend: &mut Frontend,
        db: &mut DbBackend,
        _: &mut ViewResponse,
    ) {
        if let ImageStatus::Available(texture) = frontend.image_for(self.blob_id, db) {
            ui.centered_and_justified(|ui| {
                ui.image(texture.id, texture.scaled(ui.available_size().into()));
            });
        }
    }
    fn name(&self) -> String {
        "View Image".into()
    }
}

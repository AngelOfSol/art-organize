use db::BlobId;

use crate::{
    backend::DbBackend,
    frontend::{blob, Frontend},
    views::View,
};

#[derive(Clone, Copy)]
pub struct ViewBlob {
    pub blob_id: BlobId,
}

impl View for ViewBlob {
    fn center_panel(&mut self, ui: &mut egui::Ui, frontend: &mut Frontend, db: &mut DbBackend) {
        blob::display(ui, frontend, db, self.blob_id);
    }
    fn name(&self, db: &DbBackend) -> String {
        db[self.blob_id].file_name.clone()
    }
    fn boxed_clone(&self) -> Box<dyn View> {
        Box::new(*self)
    }
}

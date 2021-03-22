use crate::{
    backend::{data_file, DbBackend},
    cli::SubCommand,
    raw_image::RawImage,
};
use db::{BlobId, Db, Piece, PieceId};
use imgui::TextureId;
use ipc::IpcReceiver;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};
use tokio::{fs, runtime::Handle, sync::mpsc};

use super::gui_state::{GuiState, MainWindow};

// pub fn request_load_image(self: &Arc<Self>, blob_id: BlobId) {
//     let this = self.clone();

//     tokio::spawn(async move {
//         let rc: Box<[u8]> = {
//             let read = this.0.read().unwrap();

//             todo!()
//             // read.db[blob_id].data.clone()
//         };

//         tokio::task::spawn_blocking(move || {
//             if let Ok((raw, thumbnail)) = RawImage::make(&rc) {
//                 this.0
//                     .read()
//                     .unwrap()
//                     .outgoing_images
//                     .blocking_send((blob_id, raw, thumbnail))
//                     .unwrap();
//             }
//         })
//         .await
//         .unwrap();
//     });
// }

pub enum ShowPieceRequest {
    Piece(PieceId),
    Blob(BlobId),
}

impl From<PieceId> for ShowPieceRequest {
    fn from(value: PieceId) -> Self {
        Self::Piece(value)
    }
}
impl From<BlobId> for ShowPieceRequest {
    fn from(value: BlobId) -> Self {
        Self::Blob(value)
    }
}

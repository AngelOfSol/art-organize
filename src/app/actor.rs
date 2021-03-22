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

pub struct Inner {
    pub ipc: IpcReceiver<SubCommand>,
    pub handle: Handle,
    pub outgoing_images: mpsc::Sender<(BlobId, RawImage, RawImage)>,

    pub gui_state: GuiState,
}

pub struct AppActor(pub RwLock<Inner>);

impl AppActor {
    #[allow(clippy::needless_lifetimes)]
    pub fn write<'a>(self: &'a Arc<Self>) -> impl std::ops::DerefMut<Target = Inner> + 'a {
        self.0.write().unwrap()
    }
    #[allow(clippy::needless_lifetimes)]
    pub fn read<'a>(self: &'a Arc<Self>) -> impl std::ops::Deref<Target = Inner> + 'a {
        self.0.read().unwrap()
    }

    pub fn request_load_image(self: &Arc<Self>, blob_id: BlobId) {
        let this = self.clone();

        tokio::spawn(async move {
            let rc: Box<[u8]> = {
                let read = this.0.read().unwrap();

                todo!()
                // read.db[blob_id].data.clone()
            };

            tokio::task::spawn_blocking(move || {
                if let Ok((raw, thumbnail)) = RawImage::make(&rc) {
                    this.0
                        .read()
                        .unwrap()
                        .outgoing_images
                        .blocking_send((blob_id, raw, thumbnail))
                        .unwrap();
                }
            })
            .await
            .unwrap();
        });
    }

    pub fn request_show_piece<I: Into<ShowPieceRequest>>(self: &Arc<Self>, id: I) {
        let id = id.into();
        let this = self.clone();

        tokio::spawn(async move {
            // let mut write = this.0.write().unwrap();

            // write.gui_state.main_window = MainWindow::Piece {
            //     id: {
            //         match id {
            //             ShowPieceRequest::Piece(id) => id,
            //             ShowPieceRequest::Blob(id) => {
            //                 match write.db.media.iter().find(|(_, blob)| *blob == id) {
            //                     Some((id, _)) => *id,
            //                     None => return,
            //                 }
            //             }
            //         }
            //     },
            //     edit: false,
            //     focused: None,
            // };
        });
    }
}

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

use crate::{backend::Backend, cli::SubCommand};
use db::{BlobId, MaybeBlob, Piece};
use imgui::TextureId;
use ipc::IpcReceiver;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};
use tokio::runtime::Handle;

pub struct Inner {
    pub ipc: IpcReceiver<SubCommand>,
    pub handle: Handle,
    pub backend: Backend,
    pub state: AppState,
    pub image_cache: BTreeMap<BlobId, TextureId>,
}

pub struct AppActor(pub RwLock<Inner>);

pub enum AppState {
    Adding { piece: Piece, blobs: Vec<MaybeBlob> },
    None,
}

impl AppActor {
    #[allow(clippy::needless_lifetimes)]
    pub fn test<'a>(self: &'a Arc<Self>) -> impl std::ops::Deref<Target = Inner> + 'a {
        self.0.read().unwrap()
    }
    pub fn request_new_piece(self: &Arc<Self>) {
        let this = self.clone();
        let handle = self.0.read().unwrap().handle.clone();

        handle.spawn(async move {
            let mut write = this.0.write().unwrap();
            write.state = AppState::Adding {
                piece: Piece::default(),
                blobs: vec![],
            }
        });
    }
}

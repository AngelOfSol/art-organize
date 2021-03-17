use crate::{backend::Backend, cli::SubCommand};
use db::{BlobId, Db, Piece};
use imgui::{im_str, TabItem, TextureId, Ui};
use ipc::IpcReceiver;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};
use tokio::runtime::Handle;

use super::piece_editor::PieceEditor;

pub struct Inner {
    pub ipc: IpcReceiver<SubCommand>,
    pub handle: Handle,
    pub backend: Backend,
    pub tabs: Vec<AppTab>,
    pub image_cache: BTreeMap<BlobId, TextureId>,
}

pub enum AppTab {
    Piece(PieceEditor),
}

pub enum TabResult {
    Keep,
    Kill,
    Selected,
}

impl AppTab {
    pub fn update(&mut self) {
        match self {
            AppTab::Piece(inner) => inner.update(),
        }
    }
    pub fn label<'a>(&self, db: &'a Db) -> Option<&'a str> {
        match self {
            AppTab::Piece(inner) => inner.label(db),
        }
    }

    pub fn render(&mut self, db: &mut Db, ui: &Ui<'_>) -> TabResult {
        let mut ret = TabResult::Keep;
        let mut open = true;

        if let Some(label) = self.label(db) {
            TabItem::new(&im_str!("{}###tab", label))
                .opened(&mut open)
                .build(ui, || {
                    ret = match self {
                        AppTab::Piece(inner) => inner.render(db, ui),
                    }
                });
        } else {
            ret = TabResult::Kill;
        }

        if !open {
            TabResult::Kill
        } else {
            ret
        }
    }
}

pub struct AppActor(pub RwLock<Inner>);

impl AppActor {
    #[allow(clippy::needless_lifetimes)]
    pub fn write<'a>(self: &'a Arc<Self>) -> impl std::ops::DerefMut<Target = Inner> + 'a {
        self.0.write().unwrap()
    }
    #[allow(clippy::needless_lifetimes)]
    pub fn _read<'a>(self: &'a Arc<Self>) -> impl std::ops::Deref<Target = Inner> + 'a {
        self.0.read().unwrap()
    }
    pub fn request_new_piece(self: &Arc<Self>) {
        let this = self.clone();

        tokio::spawn(async move {
            let mut write = this.0.write().unwrap();

            let piece = Piece::default();

            write.backend.db.undo_checkpoint();

            let id = write.backend.db.pieces.insert(piece);

            write.tabs.push(AppTab::Piece(PieceEditor { id }));
        });
    }
}

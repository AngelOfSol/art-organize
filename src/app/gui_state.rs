use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use db::{BlobId, PieceId};
use tokio::sync::mpsc;

use crate::{backend::actor::DbHandle, iter_ext, raw_image::RawImage};

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GuiState {
    pub main_window: MainWindow,

    pub search: SearchState,

    pub show_styles: bool,
    pub show_metrics: bool,
}
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SearchState {
    pub text: String,
    pub auto_complete: Vec<String>,
    pub selected: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MainWindow {
    Gallery,
    Piece {
        id: PieceId,
        edit: bool,
        focused: Option<BlobId>,
    },
}

#[derive(Debug)]
pub enum GuiAction {
    ViewPiece(PieceId),
    LoadBlob(BlobId),
    NewPiece,
    NextItem,
    PrevItem,
}

pub struct GuiHandle {
    outgoing: mpsc::UnboundedSender<GuiAction>,
}

impl GuiHandle {
    pub fn request_new_piece(&self) {
        self.outgoing.send(GuiAction::NewPiece).unwrap();
    }
    pub fn request_view_piece(&self, id: PieceId) {
        self.outgoing.send(GuiAction::ViewPiece(id)).unwrap();
    }

    pub fn request_load_image(&self, blob_id: BlobId) {
        self.outgoing.send(GuiAction::LoadBlob(blob_id)).unwrap();
    }

    pub fn next_item(&self) {
        self.outgoing.send(GuiAction::NextItem).unwrap();
    }

    pub fn prev_item(&self) {
        self.outgoing.send(GuiAction::PrevItem).unwrap();
    }
}

pub fn start_gui_task(
    db: DbHandle,
    gui_state: Arc<RwLock<GuiState>>,
    outgoing_images: mpsc::UnboundedSender<(BlobId, RawImage, RawImage)>,
) -> GuiHandle {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(gui_actor(rx, db, gui_state, outgoing_images));

    GuiHandle { outgoing: tx }
}

async fn gui_actor(
    mut incoming: mpsc::UnboundedReceiver<GuiAction>,
    db: DbHandle,
    gui_state: Arc<RwLock<GuiState>>,
    outgoing_images: mpsc::UnboundedSender<(BlobId, RawImage, RawImage)>,
) {
    while let Some(action) = incoming.recv().await {
        match action {
            GuiAction::ViewPiece(piece) => {
                let mut gui_state = gui_state.write().unwrap();
                gui_state.main_window = MainWindow::Piece {
                    id: piece,
                    edit: false,
                    focused: None,
                }
            }
            GuiAction::NewPiece => {
                let piece = db.new_piece().await.unwrap();
                let mut gui_state = gui_state.write().unwrap();
                gui_state.main_window = MainWindow::Piece {
                    id: piece,
                    edit: false,
                    focused: None,
                }
            }
            GuiAction::LoadBlob(blob_id) => {
                let rc = {
                    let read = db.read().unwrap();

                    read[blob_id].data.clone()
                };
                let outgoing_images = outgoing_images.clone();

                tokio::task::spawn_blocking(move || {
                    if let Ok((raw, thumbnail)) = RawImage::make(&rc) {
                        outgoing_images.send((blob_id, raw, thumbnail)).unwrap();
                    }
                });
            }
            GuiAction::NextItem => {
                let db = db.read().unwrap();
                let mut gui_state = gui_state.write().unwrap();
                if let MainWindow::Piece { id, focused, .. } = &mut gui_state.main_window {
                    if let Some(blob_id) = focused {
                        if let Some(new) = iter_ext::next(
                            db.blobs_for_piece(*id)
                                .filter(|blob| db[*blob].blob_type == db[*blob_id].blob_type),
                            *blob_id,
                        ) {
                            *focused = Some(new);
                        }
                    }
                }
            }
            GuiAction::PrevItem => {
                let db = db.read().unwrap();
                let mut gui_state = gui_state.write().unwrap();
                if let MainWindow::Piece { id, focused, .. } = &mut gui_state.main_window {
                    if let Some(blob_id) = focused {
                        if let Some(new) = iter_ext::prev(
                            db.blobs_for_piece(*id)
                                .filter(|blob| db[*blob].blob_type == db[*blob_id].blob_type),
                            *blob_id,
                        ) {
                            *focused = Some(new);
                        }
                    }
                }
            }
        }
    }
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::Gallery
    }
}

impl Display for MainWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MainWindow::Gallery => "Gallery",
                MainWindow::Piece { .. } => "Piece",
            }
        )
    }
}

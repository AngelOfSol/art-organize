use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use db::{BlobId, PieceId};
use tokio::sync::mpsc;

use crate::{backend::actor::DbHandle, raw_image::RawImage};

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
}

pub fn start_gui_task(
    db: DbHandle,
    gui_state: Arc<RwLock<GuiState>>,
    outgoing_images: mpsc::UnboundedSender<(BlobId, RawImage, RawImage)>,
) -> GuiHandle {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(gui_actor(rx, db.clone(), gui_state, outgoing_images));

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

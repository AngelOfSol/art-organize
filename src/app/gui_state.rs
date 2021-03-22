use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use db::{BlobId, PieceId};
use tokio::sync::mpsc;

use crate::backend::actor::DbHandle;

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
    NewPiece,
}

pub struct GuiHandle {
    gui_state: Arc<RwLock<GuiState>>,
    outgoing: mpsc::UnboundedSender<GuiAction>,
}

impl GuiHandle {
    pub fn request_new_piece(&self) {
        self.outgoing.send(GuiAction::NewPiece).unwrap();
    }
    pub fn request_view_piece(&self, id: PieceId) {
        self.outgoing.send(GuiAction::ViewPiece(id)).unwrap();
    }
}

async fn gui_state(
    mut incoming: mpsc::UnboundedReceiver<GuiAction>,
    db: DbHandle,
    gui_state: Arc<RwLock<GuiState>>,
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

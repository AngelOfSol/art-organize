use std::fmt::Display;

use db::{BlobId, PieceId};

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

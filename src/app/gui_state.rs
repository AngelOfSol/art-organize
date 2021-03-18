use std::fmt::Display;

use db::BlobId;

#[derive(Default)]
pub struct GuiState {
    pub main_window: MainWindow,
}

pub enum MainWindow {
    Gallery,
    Blob { id: BlobId },
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
                MainWindow::Blob { .. } => "Image",
            }
        )
    }
}

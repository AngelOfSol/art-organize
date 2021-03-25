use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    fs::File,
    io::BufReader,
    ops::Deref,
    sync::{Arc, RwLock},
};

use db::{BlobId, PieceId};
use tokio::sync::mpsc;

use crate::{
    backend::actor::DbHandle,
    iter_ext,
    raw_image::{RawImage, TextureImage},
};

pub mod blob;
pub mod gallery;
pub mod piece;

#[derive(Default, Clone, PartialEq, Eq)]
pub struct GuiState {
    pub main_window: MainWindow,
    pub inner: InnerGuiState,
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct InnerGuiState {
    pub show_styles: bool,
    pub show_metrics: bool,

    pub search: SearchState,
    pub thumbnails: BTreeMap<BlobId, TextureImage>,
    pub images: BTreeMap<BlobId, TextureImage>,

    requested: BTreeSet<BlobId>,
}

pub struct StateRef<'a> {
    pub search: &'a SearchState,
    pub thumbnails: &'a BTreeMap<BlobId, TextureImage>,
    pub images: &'a BTreeMap<BlobId, TextureImage>,
}

pub trait GuiView {
    fn draw_main(&mut self, gui_handle: &GuiHandle, gui_state: &InnerGuiState, ui: &imgui::Ui<'_>);
    fn draw_explorer(
        &mut self,
        gui_handle: &GuiHandle,
        gui_state: &InnerGuiState,
        ui: &imgui::Ui<'_>,
    );
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
    RequestImage(BlobId),
    ImageCreated {
        blob_id: BlobId,
        image: TextureImage,
        is_thumbnail: bool,
    },
    NewPiece,
    NextItem,
    PrevItem,
}

pub struct GuiHandle {
    outgoing: mpsc::UnboundedSender<GuiAction>,
    db: DbHandle,
}

impl Deref for GuiHandle {
    type Target = DbHandle;
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl GuiHandle {
    pub fn request_new_piece(&self) {
        self.outgoing.send(GuiAction::NewPiece).unwrap();
    }
    pub fn request_view_piece(&self, id: PieceId) {
        self.outgoing.send(GuiAction::ViewPiece(id)).unwrap();
    }

    pub fn request_load_image(&self, blob_id: BlobId) {
        self.outgoing
            .send(GuiAction::RequestImage(blob_id))
            .unwrap();
    }

    pub fn forward_image(&self, blob_id: BlobId, image: TextureImage, is_thumbnail: bool) {
        self.outgoing
            .send(GuiAction::ImageCreated {
                blob_id,
                image,
                is_thumbnail,
            })
            .unwrap();
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
    outgoing_images: mpsc::UnboundedSender<(BlobId, RawImage, bool)>,
) -> GuiHandle {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(gui_actor(rx, db.clone(), gui_state, outgoing_images));

    GuiHandle { outgoing: tx, db }
}

async fn gui_actor(
    mut incoming: mpsc::UnboundedReceiver<GuiAction>,
    db: DbHandle,
    gui_state: Arc<RwLock<GuiState>>,
    outgoing_images: mpsc::UnboundedSender<(BlobId, RawImage, bool)>,
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
            GuiAction::RequestImage(blob_id) => {
                let read = db.read().unwrap();

                {
                    let mut gui_state = gui_state.write().unwrap();
                    if gui_state.inner.requested.contains(&blob_id) {
                        continue;
                    } else {
                        gui_state.inner.requested.insert(blob_id);
                    }
                }

                let hash = read[blob_id].hash;
                let storage = read.storage_for(blob_id);

                let outgoing_images = outgoing_images.clone();

                let gui_state = gui_state.clone();

                tokio::task::spawn_blocking(move || {
                    let test = File::open(&storage)
                        .map_err(|err| err.into())
                        .and_then(|file| {
                            let file = BufReader::new(file);
                            RawImage::make(file, hash)
                        });

                    match test {
                        Ok((raw, thumb)) => {
                            outgoing_images.send((blob_id, thumb, true)).unwrap();
                            outgoing_images.send((blob_id, raw, false)).unwrap();
                        }
                        Err(_) => {
                            let mut gui_state = gui_state.write().unwrap();
                            gui_state.inner.requested.remove(&blob_id);
                        }
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
                                .filter(|blob| db[blob].blob_type == db[*blob_id].blob_type),
                            *blob_id,
                        ) {
                            *focused = Some(new);
                        }
                    }
                }
            }
            GuiAction::ImageCreated {
                blob_id,
                image,
                is_thumbnail,
            } => {
                let mut gui_state = gui_state.write().unwrap();
                if is_thumbnail {
                    gui_state.inner.thumbnails.insert(blob_id, image);
                } else {
                    gui_state.inner.images.insert(blob_id, image);
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

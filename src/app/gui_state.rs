use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::{mpsc as std_mpsc, Arc, RwLock},
};

use db::BlobId;
use home::Home;
use tokio::sync::mpsc;

use crate::{
    backend::actor::DbHandle,
    raw_image::{RawImage, TextureImage},
};

use self::{gallery::Gallery, piece::PieceView};

pub mod blob;
pub mod gallery;
pub mod help;
pub mod home;
pub mod piece;
pub mod update;

pub struct GuiState {
    view_stack: Vec<Box<dyn GuiView>>,
    inner: InnerGuiState,
}

impl Deref for GuiState {
    type Target = InnerGuiState;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for GuiState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl GuiState {
    pub fn update(&self, gui_handle: &GuiHandle) {
        self.view_stack.last().unwrap().update(gui_handle);
    }

    pub fn render_main(&mut self, gui_handle: &GuiHandle, ui: &imgui::Ui<'_>) {
        self.view_stack
            .last_mut()
            .unwrap()
            .draw_main(gui_handle, &self.inner, ui)
    }
    pub fn render_explorer(&mut self, gui_handle: &GuiHandle, ui: &imgui::Ui<'_>) {
        self.view_stack
            .last_mut()
            .unwrap()
            .draw_explorer(gui_handle, &self.inner, ui)
    }
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            view_stack: vec![Box::new(Home)],
            inner: InnerGuiState::default(),
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct InnerGuiState {
    pub show_styles: bool,
    pub show_metrics: bool,

    pub search: SearchState,
    pub thumbnails: BTreeMap<BlobId, TextureImage>,
    pub images: BTreeMap<BlobId, TextureImage>,

    pub requested: BTreeSet<BlobId>,
}

impl InnerGuiState {
    pub fn invalidate(&mut self, id: &BlobId) {
        self.images.remove(id);
        self.thumbnails.remove(id);
        self.requested.remove(id);
    }
}

pub trait GuiView: Sync + Send + Debug {
    fn update(&self, gui_handle: &GuiHandle);

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

#[derive(Debug)]
pub enum GuiAction {
    RequestImage(BlobId),
    ImageCreated {
        blob_id: BlobId,
        image: TextureImage,
        is_thumbnail: bool,
    },
    NewPiece,
    Back,
    Push(Box<dyn GuiView>),
    NewDB,
    LoadDB,
}

#[derive(Clone)]
pub struct GuiActionHandle {
    outgoing: mpsc::UnboundedSender<GuiAction>,
    db: DbHandle,
}

impl Deref for GuiActionHandle {
    type Target = DbHandle;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

pub struct GuiHandle {
    outgoing: GuiActionHandle,
    pub incoming_files: std_mpsc::Receiver<PathBuf>,
}

impl Deref for GuiHandle {
    type Target = GuiActionHandle;
    fn deref(&self) -> &Self::Target {
        &self.outgoing
    }
}

impl GuiActionHandle {
    pub fn request_new_piece(&self) {
        self.outgoing.send(GuiAction::NewPiece).unwrap();
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

    pub fn go_back(&self) {
        self.outgoing.send(GuiAction::Back).unwrap();
    }

    pub fn goto<V: GuiView + 'static>(&self, state: V) {
        self.outgoing
            .send(GuiAction::Push(Box::new(state)))
            .unwrap();
    }

    pub fn new_db(&self) {
        self.outgoing.send(GuiAction::NewDB).unwrap();
    }
    pub fn load_db(&self) {
        self.outgoing.send(GuiAction::LoadDB).unwrap();
    }
}

pub fn start_gui_task(
    db: DbHandle,
    gui_state: Arc<RwLock<GuiState>>,
    outgoing_images: std_mpsc::Sender<(BlobId, RawImage, bool)>,
    incoming_files: std_mpsc::Receiver<PathBuf>,
) -> GuiHandle {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(gui_actor(rx, db.clone(), gui_state, outgoing_images));

    GuiHandle {
        outgoing: GuiActionHandle { outgoing: tx, db },
        incoming_files,
    }
}

async fn gui_actor(
    mut incoming: mpsc::UnboundedReceiver<GuiAction>,
    db: DbHandle,
    gui_state: Arc<RwLock<GuiState>>,
    outgoing_images: std_mpsc::Sender<(BlobId, RawImage, bool)>,
) {
    while let Some(action) = incoming.recv().await {
        match action {
            GuiAction::NewPiece => {
                let id = db.new_piece().await.unwrap();
                let mut gui_state = gui_state.write().unwrap();

                gui_state
                    .view_stack
                    .push(Box::new(PieceView { id, edit: false }));
            }
            GuiAction::RequestImage(blob_id) => {
                let outgoing_images = outgoing_images.clone();
                let db = db.clone();
                let gui_state = gui_state.clone();
                tokio::task::spawn_blocking(move || {
                    let read = db.read().unwrap();

                    let hash = read[blob_id].hash;
                    {
                        let mut gui_state = gui_state.write().unwrap();
                        if gui_state.inner.requested.contains(&blob_id) {
                            return;
                        }
                        gui_state.inner.requested.insert(blob_id);
                    }

                    let storage = read.storage_for(blob_id);

                    let gui_state = gui_state.clone();

                    let test = RawImage::make(storage, hash);

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
            GuiAction::Back => {
                let mut gui_state = gui_state.write().unwrap();
                if gui_state.view_stack.len() > 1 {
                    gui_state.view_stack.pop();
                }
            }
            GuiAction::Push(state) => {
                let mut gui_state = gui_state.write().unwrap();
                gui_state.view_stack.push(state);
            }

            GuiAction::NewDB => {
                db.new_db();
                let mut gui_state = gui_state.write().unwrap();
                *gui_state = GuiState::default();
                gui_state.view_stack.push(Box::new(Gallery));
            }
            GuiAction::LoadDB => {
                db.load_db();
                let mut gui_state = gui_state.write().unwrap();
                *gui_state = GuiState::default();
                gui_state.view_stack.push(Box::new(Gallery));
            }
        }
    }
}

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

use self::{category::CategoryView, gallery::Gallery, piece::PieceView, tag::TagView};

pub mod blob;
pub mod category;
pub mod gallery;
pub mod help;
pub mod home;
pub mod piece;
pub mod piece_list;
pub mod search;
pub mod tag;
pub mod tag_list;
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
    pub fn label(&self) -> &'static str {
        self.view_stack.last().unwrap().label()
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

    pub requested: BTreeSet<(BlobId, bool)>,
}

impl InnerGuiState {
    pub fn invalidate(&mut self, id: &BlobId) {
        self.images.remove(id);
        self.thumbnails.remove(id);
        self.requested.remove(&(*id, false));
        self.requested.remove(&(*id, true));
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

    fn label(&self) -> &'static str;
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SearchState {
    pub text: String,
    pub auto_complete: Vec<String>,
    pub selected: Option<usize>,
}

#[derive(Debug)]
pub enum GuiAction {
    RequestImage(BlobId, bool),
    ImageCreated {
        blob_id: BlobId,
        image: TextureImage,
        is_thumbnail: bool,
    },
    NewPiece,
    NewTag,
    NewCategory,
    Back,
    Push(Box<dyn GuiView>),
    NewDB,
    LoadDB,
    CopyToClipboard(BlobId),
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
    pub fn request_new_tag(&self) {
        self.outgoing.send(GuiAction::NewTag).unwrap();
    }
    pub fn request_new_category(&self) {
        self.outgoing.send(GuiAction::NewCategory).unwrap();
    }

    pub fn request_load_image_borked(&self, blob_id: BlobId) {
        self.outgoing
            .send(GuiAction::RequestImage(blob_id, false))
            .unwrap();
    }
    pub fn request_load_thumbnail(&self, blob_id: BlobId) {
        self.outgoing
            .send(GuiAction::RequestImage(blob_id, true))
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

    pub fn copy_to_clipboard(&self, blob_id: BlobId) {
        self.outgoing
            .send(GuiAction::CopyToClipboard(blob_id))
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
                    .push(Box::new(PieceView { id, edit: true }));
            }
            GuiAction::NewTag => {
                let id = db.new_tag().await.unwrap();
                let mut gui_state = gui_state.write().unwrap();

                gui_state
                    .view_stack
                    .push(Box::new(TagView { id, edit: true }));
            }
            GuiAction::NewCategory => {
                let id = db.new_category().await.unwrap();
                let mut gui_state = gui_state.write().unwrap();

                gui_state
                    .view_stack
                    .push(Box::new(CategoryView { id, edit: true }));
            }
            GuiAction::RequestImage(blob_id, thumbnail) => {
                let outgoing_images = outgoing_images.clone();
                let db = db.clone();
                let gui_state = gui_state.clone();

                let check_req = gui_state.read().unwrap();
                if !check_req.requested.contains(&(blob_id, thumbnail)) {
                    drop(check_req);
                    tokio::task::spawn_blocking(move || {
                        let read = db.read().unwrap();

                        let hash = read[blob_id].hash;
                        {
                            let mut gui_state = gui_state.write().unwrap();
                            if gui_state.inner.requested.contains(&(blob_id, thumbnail)) {
                                return;
                            }
                            gui_state.inner.requested.insert((blob_id, thumbnail));
                        }

                        let storage = read.storage_for(blob_id);

                        let gui_state = gui_state.clone();

                        let test = RawImage::make(storage, hash);

                        match test {
                            Ok((raw, thumb)) => {
                                if thumbnail {
                                    outgoing_images.send((blob_id, thumb, true)).unwrap();
                                } else {
                                    outgoing_images.send((blob_id, raw, false)).unwrap();
                                }
                            }
                            Err(_) => {
                                let mut gui_state = gui_state.write().unwrap();
                                gui_state.inner.requested.remove(&(blob_id, thumbnail));
                            }
                        }
                    });
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
            GuiAction::CopyToClipboard(blob_id) => {
                let db = db.clone();

                tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                    let db = db.read().unwrap();
                    let storage = db.storage_for(blob_id);
                    let image = image::DynamicImage::ImageRgb8(
                        image::io::Reader::open(&storage)?
                            .with_guessed_format()?
                            .decode()?
                            .to_rgb8(),
                    );
                    let mut bitmap = Vec::<u8>::new();

                    image.write_to(&mut bitmap, image::ImageOutputFormat::Bmp)?;

                    clipboard_win::set_clipboard(clipboard_win::formats::Bitmap, &bitmap).unwrap();
                    Ok(())
                });
            }
        }
    }
}

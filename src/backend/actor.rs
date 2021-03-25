use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::Deref,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use chrono::Local;
use db::{
    commands::{AttachBlob, EditPiece},
    Blob, BlobType, Db, Piece, PieceId,
};
use futures_util::{stream::FuturesUnordered, StreamExt};
use tokio::{
    fs,
    sync::{mpsc, oneshot},
};

use super::{data_file, DbBackend};

#[derive(Debug, Clone)]
pub struct DbHandle {
    backend: Arc<RwLock<DbBackend>>,
    outgoing: mpsc::UnboundedSender<AppAction>,
}

impl Deref for DbHandle {
    type Target = Arc<RwLock<DbBackend>>;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl DbHandle {
    pub fn undo(&self) {
        self.outgoing.send(AppAction::Undo).unwrap();
    }
    pub fn redo(&self) {
        self.outgoing.send(AppAction::Redo).unwrap();
    }
    pub fn update_piece(&self, data: EditPiece) {
        self.outgoing
            .send(AppAction::Db(DbAction::EditPiece(data)))
            .unwrap();
    }
    pub fn new_piece(&self) -> oneshot::Receiver<PieceId> {
        let (tx, rx) = oneshot::channel();
        self.outgoing
            .send(AppAction::Db(DbAction::NewPiece(tx)))
            .unwrap();
        rx
    }
    pub fn ask_blobs_for_piece(&self, to: PieceId, blob_type: BlobType) {
        self.outgoing
            .send(AppAction::Db(DbAction::AskBlobs { to, blob_type }))
            .unwrap();
    }
    pub fn new_blob_from_file(&self, to: PieceId, blob_type: BlobType, path: PathBuf) {
        self.outgoing
            .send(AppAction::Db(DbAction::AddBlob {
                to,
                blob_type,
                path,
            }))
            .unwrap();
    }
}

#[derive(Debug)]
pub enum AppAction {
    Undo,
    Redo,
    Db(DbAction),
}

#[derive(Debug)]
pub enum DbAction {
    NewPiece(oneshot::Sender<PieceId>),
    EditPiece(EditPiece),
    AskBlobs {
        to: PieceId,
        blob_type: BlobType,
    },
    AddBlob {
        to: PieceId,
        blob_type: BlobType,
        path: PathBuf,
    },
}

pub fn start_db_task(backend: Arc<RwLock<DbBackend>>) -> DbHandle {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(db_actor(rx, backend.clone()));

    DbHandle {
        backend,
        outgoing: tx,
    }
}

async fn db_actor(mut incoming: mpsc::UnboundedReceiver<AppAction>, data: Arc<RwLock<DbBackend>>) {
    while let Some(action) = incoming.recv().await {
        match action {
            AppAction::Undo => {
                let mut db = data.write().unwrap();
                db.undo();
            }
            AppAction::Redo => {
                let mut db = data.write().unwrap();
                db.redo();
            }
            AppAction::Db(DbAction::AskBlobs { to, blob_type }) => {
                let data = data.clone();
                tokio::spawn(async move {
                    let files = if let Some(files) = rfd::AsyncFileDialog::new().pick_files().await
                    {
                        files
                    } else {
                        return;
                    };

                    let file_futures: FuturesUnordered<_> = files
                        .into_iter()
                        .map(|file| async move {
                            let file = file.path().to_path_buf();
                            let raw_data = fs::read(&file).await.unwrap();
                            let mut hash = DefaultHasher::new();
                            raw_data.hash(&mut hash);
                            let hash = hash.finish();

                            Blob {
                                file_name: file.to_string_lossy().into(),
                                hash,
                                data: Arc::new(raw_data),
                                blob_type,
                                added: Local::now(),
                            }
                        })
                        .collect();
                    let files: Vec<_> = file_futures.collect().await;

                    {
                        let mut db = data.write().unwrap();
                        db.undo_checkpoint();

                        for blob in files {
                            let id = db.create_blob(blob);
                            db.attach_blob(AttachBlob { src: to, dest: id });
                        }
                    }
                    save_data(&data).await;
                });
            }
            AppAction::Db(DbAction::AddBlob {
                to,
                blob_type,
                path,
            }) => {
                let data = data.clone();
                tokio::spawn(async move {
                    let raw_data = fs::read(&path).await.unwrap();
                    let mut hash = DefaultHasher::new();
                    raw_data.hash(&mut hash);
                    let hash = hash.finish();

                    let blob = Blob {
                        file_name: path.to_string_lossy().into(),
                        hash,
                        data: Arc::new(raw_data),
                        blob_type,
                        added: Local::now(),
                    };

                    {
                        let mut db = data.write().unwrap();
                        db.undo_checkpoint();

                        let id = db.create_blob(blob);
                        db.attach_blob(AttachBlob { src: to, dest: id });
                    }
                    save_data(&data).await;
                });
            }
            AppAction::Db(db_action) => {
                let mut db = data.write().unwrap();
                db.undo_checkpoint();

                match db_action {
                    DbAction::EditPiece(edit) => {
                        db.edit_piece(edit);
                    }
                    DbAction::NewPiece(sender) => {
                        let id = db.create_piece(Piece::default());
                        sender.send(id).unwrap();
                    }
                    DbAction::AskBlobs { .. } | DbAction::AddBlob { .. } => unreachable!(),
                }
            }
        }
        save_data(&data).await;
    }
}

async fn save_data(data: &Arc<RwLock<DbBackend>>) {
    let (root, data) = {
        let db = data.read().unwrap();
        let root = data_file(db.root.clone());
        let data = bincode::serialize::<Db>(&db).unwrap();

        (root, data)
    };
    fs::write(root, &data).await.unwrap();
}

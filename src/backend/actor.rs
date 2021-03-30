use std::{
    ops::Deref,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use db::{
    commands::{AttachBlob, EditBlob, EditPiece},
    BlobId, BlobType, Db, Piece, PieceId,
};
use futures_util::{stream::FuturesUnordered, StreamExt};
use tokio::{
    fs,
    sync::{mpsc, oneshot},
};
mod blob {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
        path::PathBuf,
    };

    use chrono::Local;
    use db::{Blob, BlobType};

    pub async fn from_path(path: PathBuf, blob_type: BlobType) -> anyhow::Result<Blob> {
        let raw_data = tokio::fs::read(&path).await?;
        let mut hash = DefaultHasher::new();
        raw_data.hash(&mut hash);
        let hash = hash.finish();

        Ok(Blob {
            file_name: path.file_name().unwrap().to_string_lossy().into_owned(),
            hash,
            blob_type,
            added: Local::today().naive_local(),
        })
    }
}

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

    pub fn new_piece(&self) -> oneshot::Receiver<PieceId> {
        let (tx, rx) = oneshot::channel();
        self.outgoing
            .send(AppAction::Db(DbAction::NewPiece(tx)))
            .unwrap();
        rx
    }

    pub fn update_piece(&self, data: EditPiece) {
        self.outgoing
            .send(AppAction::Db(DbAction::EditPiece(data)))
            .unwrap();
    }
    pub fn update_blob(&self, data: EditBlob) {
        self.outgoing
            .send(AppAction::Db(DbAction::EditBlob(data)))
            .unwrap();
    }

    pub fn delete_piece(&self, id: PieceId) {
        self.outgoing
            .send(AppAction::Db(DbAction::DeletePiece(id)))
            .unwrap();
    }
    pub fn delete_blob(&self, id: BlobId) {
        self.outgoing
            .send(AppAction::Db(DbAction::DeleteBlob(id)))
            .unwrap();
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
    EditBlob(EditBlob),
    DeletePiece(PieceId),
    DeleteBlob(BlobId),
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
                let db = data.read().unwrap();
                if !db.exists(to) {
                    continue;
                }

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

                            Ok::<_, anyhow::Error>((
                                file.clone(),
                                blob::from_path(file, blob_type).await?,
                            ))
                        })
                        .collect();
                    let files: Vec<_> = file_futures.collect().await;

                    let mut out_futures = FuturesUnordered::new();
                    {
                        let mut db = data.write().unwrap();
                        db.undo_checkpoint();

                        for (path, blob) in files.into_iter().filter_map(Result::ok) {
                            let id = db.create_blob(blob);

                            db.attach_blob(AttachBlob { src: to, dest: id });
                            out_futures.push(fs::copy(path, db.storage_for(id)));
                        }
                    }
                    while let Some(result) = out_futures.next().await {
                        result.unwrap();
                    }

                    save_data(&data).await;
                });
            }
            AppAction::Db(DbAction::AddBlob {
                to,
                blob_type,
                path,
            }) => {
                let db = data.read().unwrap();
                if !db.exists(to) {
                    continue;
                }

                let data = data.clone();
                tokio::spawn(async move {
                    let blob = blob::from_path(path.clone(), blob_type).await.unwrap();

                    let storage = {
                        let mut db = data.write().unwrap();
                        db.undo_checkpoint();

                        let id = db.create_blob(blob);

                        db.attach_blob(AttachBlob { src: to, dest: id });
                        db.storage_for(id)
                    };
                    fs::copy(path, storage).await.unwrap();
                    save_data(&data).await;
                });
            }
            AppAction::Db(db_action) => {
                let mut db = data.write().unwrap();
                db.undo_checkpoint();

                match db_action {
                    DbAction::EditPiece(edit) => {
                        // TODO log falses here
                        db.edit(edit);
                    }
                    DbAction::EditBlob(edit) => {
                        let from = db.storage_for(edit.id);

                        let id = edit.id;
                        db.edit(edit);

                        let to = db.storage_for(id);
                        if from != to {
                            std::fs::copy(&from, to).unwrap();
                            std::fs::remove_file(from).unwrap();
                        }
                    }
                    DbAction::NewPiece(sender) => {
                        let id = db.create_piece(Piece::default());
                        sender.send(id).unwrap();
                    }
                    DbAction::DeletePiece(id) => {
                        assert!(db.delete(id));
                    }
                    DbAction::DeleteBlob(id) => {
                        assert!(db.delete(id));
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

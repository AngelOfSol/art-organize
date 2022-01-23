use std::{
    ffi::OsStr,
    ops::Deref,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use db::{
    v2::commands::{
        AttachBlob, AttachCategory, AttachTag, EditBlob, EditCategory, EditPiece, EditTag,
    },
    v2::DbV2 as Db,
    v2::Piece,
    v2::PieceId,
    BlobId, BlobType, Category, CategoryId, Tag, TagId,
};
use futures_util::{stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use regex::Regex;
use rfd::AsyncFileDialog;
use tokio::{
    fs,
    sync::{mpsc, oneshot, watch},
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

use crate::config::Config;

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

    pub fn new_db(&self) {
        self.outgoing.send(AppAction::NewDB).unwrap();
    }

    pub fn load_db(&self) {
        self.outgoing.send(AppAction::LoadDB).unwrap();
    }

    pub fn new_piece(&self) -> oneshot::Receiver<PieceId> {
        let (tx, rx) = oneshot::channel();
        self.outgoing
            .send(AppAction::Db(DbAction::NewPiece(tx)))
            .unwrap();
        rx
    }

    pub fn new_tag(&self) -> oneshot::Receiver<TagId> {
        let (tx, rx) = oneshot::channel();
        self.outgoing
            .send(AppAction::Db(DbAction::NewTag(tx)))
            .unwrap();
        rx
    }
    pub fn new_category(&self) -> oneshot::Receiver<CategoryId> {
        let (tx, rx) = oneshot::channel();
        self.outgoing
            .send(AppAction::Db(DbAction::NewCategory(tx)))
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
    pub fn update_tag(&self, data: EditTag) {
        self.outgoing
            .send(AppAction::Db(DbAction::EditTag(data)))
            .unwrap();
    }
    pub fn update_category(&self, data: EditCategory) {
        self.outgoing
            .send(AppAction::Db(DbAction::EditCategory(data)))
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
    pub fn delete_tag(&self, id: TagId) {
        self.outgoing
            .send(AppAction::Db(DbAction::DeleteTag(id)))
            .unwrap();
    }
    pub fn delete_category(&self, id: CategoryId) {
        self.outgoing
            .send(AppAction::Db(DbAction::DeleteCategory(id)))
            .unwrap();
    }

    pub fn attach_category(&self, attach: AttachCategory) {
        self.outgoing
            .send(AppAction::Db(DbAction::AttachCategory(attach)))
            .unwrap();
    }
    pub fn attach_tag(&self, attach: AttachTag) {
        self.outgoing
            .send(AppAction::Db(DbAction::AttachTag(attach)))
            .unwrap();
    }
    pub fn remove_tag(&self, remove: AttachTag) {
        self.outgoing
            .send(AppAction::Db(DbAction::RemoveTag(remove)))
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

    pub fn clean_blobs(&self) {
        self.outgoing
            .send(AppAction::Db(DbAction::CleanBlobs))
            .unwrap();
    }
    pub fn save_to_file(&self, id: BlobId) {
        self.outgoing
            .send(AppAction::Db(DbAction::SaveBlobToFile(id)))
            .unwrap();
    }
}

#[derive(Debug)]
pub enum AppAction {
    Undo,
    Redo,
    NewDB,
    LoadDB,
    Db(DbAction),
}

#[derive(Debug)]
pub enum DbAction {
    NewPiece(oneshot::Sender<PieceId>),
    NewTag(oneshot::Sender<TagId>),
    NewCategory(oneshot::Sender<CategoryId>),
    EditPiece(EditPiece),
    EditBlob(EditBlob),
    EditTag(EditTag),
    EditCategory(EditCategory),
    DeletePiece(PieceId),
    DeleteBlob(BlobId),
    DeleteTag(TagId),
    DeleteCategory(CategoryId),
    AttachCategory(AttachCategory),
    AttachTag(AttachTag),
    RemoveTag(AttachTag),
    AskBlobs {
        to: PieceId,
        blob_type: BlobType,
    },
    AddBlob {
        to: PieceId,
        blob_type: BlobType,
        path: PathBuf,
    },
    CleanBlobs,
    SaveBlobToFile(BlobId),
}

pub fn start_db_task(backend: Arc<RwLock<DbBackend>>) -> DbHandle {
    let (tx, rx) = mpsc::unbounded_channel();
    let (send_dirty, recv_dirty) = watch::channel(());

    tokio::spawn(db_actor(rx, Arc::new(send_dirty), backend.clone()));
    tokio::spawn(save_db_actor(recv_dirty, backend.clone()));

    DbHandle {
        backend,
        outgoing: tx,
    }
}

async fn db_actor(
    mut incoming: mpsc::UnboundedReceiver<AppAction>,
    dirty: Arc<watch::Sender<()>>,
    data: Arc<RwLock<DbBackend>>,
) {
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
            AppAction::NewDB => {
                let root = if let Some(file) = AsyncFileDialog::new().pick_folder().await {
                    file.path().to_path_buf()
                } else {
                    continue;
                };
                if let Ok(new) = DbBackend::init_at_directory(root.clone()).await {
                    let mut db = data.write().unwrap();
                    let mut config = Config::load().unwrap();
                    config.default_dir = Some(root);
                    config.save().unwrap();
                    *db = new;
                } else {
                    continue;
                };
            }
            AppAction::LoadDB => {
                let mut root = if let Some(file) = AsyncFileDialog::new()
                    .add_filter("ArtOrganize Database", &["aodb"])
                    .pick_file()
                    .await
                {
                    file.path().to_path_buf()
                } else {
                    continue;
                };
                if let Ok(new) = DbBackend::from_file(root.clone()).await {
                    let mut db = data.write().unwrap();
                    let mut config = Config::load().unwrap();
                    root.pop();
                    config.default_dir = Some(root);
                    config.save().unwrap();
                    *db = new;
                } else {
                    continue;
                };
            }
            AppAction::Db(DbAction::AskBlobs { to, blob_type }) => {
                let db = data.read().unwrap();
                if !db.exists(to) {
                    continue;
                }

                let data = data.clone();
                let dirty = dirty.clone();

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

                    dirty.send(()).unwrap();
                });
            }

            AppAction::Db(DbAction::SaveBlobToFile(id)) => {
                let db = data.read().unwrap();
                if !db.exists(id) {
                    continue;
                }

                let (storage, file_name) = {
                    let db = data.read().unwrap();
                    (db.storage_for(id), db[id].file_name.clone())
                };
                let dirty = dirty.clone();

                tokio::spawn(async move {
                    let mut dialog = rfd::AsyncFileDialog::new().set_file_name(&file_name);
                    if let Some(ext) = Path::new(&file_name)
                        .extension()
                        .and_then(|inner| inner.to_str())
                    {
                        dialog = dialog.add_filter("Image", &[ext]);
                    }
                    let file = if let Some(files) = dialog.save_file().await {
                        files
                    } else {
                        return;
                    };

                    fs::copy(storage, file.path()).await.unwrap();

                    dirty.send(()).unwrap();
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
                let dirty = dirty.clone();
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
                    dirty.send(()).unwrap();
                });
            }
            AppAction::Db(DbAction::CleanBlobs) => {
                {
                    let mut db = data.write().unwrap();
                    let dangling_blobs = db
                        .blobs()
                        .map(|(id, _)| id)
                        .filter(|blob_id| db.pieces_for_blob(*blob_id).count() == 0)
                        .collect_vec();
                    for blob_id in dangling_blobs {
                        db.delete(blob_id);
                    }
                }

                let (paths, root) = {
                    let db = data.read().unwrap();
                    (
                        db.blobs().map(|(id, _)| db.storage_for(id)).collect_vec(),
                        db.root.clone(),
                    )
                };
                let mut root = fs::read_dir(root).await.unwrap();

                let regex = Regex::new(r"^\[\d+\] ").unwrap();

                let mut to_remove = FuturesUnordered::new();

                while let Ok(Some(entry)) = root.next_entry().await {
                    let file_path = entry.path();
                    if let Some(file_name) = file_path.file_name().and_then(OsStr::to_str) {
                        if paths.iter().all(|item| item != &file_path) && regex.is_match(file_name)
                        {
                            to_remove.push(tokio::task::spawn_blocking(move || {
                                trash::delete(file_path)
                            }));
                        }
                    }
                }
                while let Some(item) = to_remove.next().await {
                    item.unwrap().unwrap();
                }
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
                    DbAction::EditTag(edit) => {
                        // TODO log falses here
                        db.edit(edit);
                    }
                    DbAction::EditCategory(edit) => {
                        // TODO log falses here
                        db.edit(edit);
                    }
                    DbAction::NewPiece(sender) => {
                        let id = db.create_piece(Piece::default());
                        sender.send(id).unwrap();
                    }
                    DbAction::NewTag(sender) => {
                        let id = db.create_tag(Tag::default());
                        sender.send(id).unwrap();
                    }
                    DbAction::NewCategory(sender) => {
                        let id = db.create_category(Category::default());
                        sender.send(id).unwrap();
                    }
                    DbAction::DeletePiece(id) => {
                        assert!(db.delete(id));
                    }
                    DbAction::DeleteBlob(id) => {
                        assert!(db.delete(id));
                    }
                    DbAction::DeleteTag(id) => {
                        assert!(db.delete(id));
                    }
                    DbAction::DeleteCategory(id) => {
                        assert!(db.delete(id));
                    }
                    DbAction::AttachCategory(attach) => {
                        assert!(db.attach_category(attach));
                    }
                    DbAction::AttachTag(attach) => {
                        assert!(db.attach_tag(attach));
                    }
                    DbAction::RemoveTag(remove) => {
                        assert!(db.remove_tag(remove));
                    }
                    DbAction::AskBlobs { .. }
                    | DbAction::AddBlob { .. }
                    | DbAction::CleanBlobs
                    | DbAction::SaveBlobToFile(_) => {
                        unreachable!()
                    }
                }
            }
        }
        dirty.send(()).unwrap();
    }
}

async fn save_db_actor(
    mut dirty: watch::Receiver<()>,
    data: Arc<RwLock<DbBackend>>,
) -> anyhow::Result<()> {
    loop {
        dirty.changed().await?;
        let (root, data) = {
            let db = data.read().unwrap();
            let root = data_file(db.root.clone());
            let data = bincode::serialize::<Db>(&db).unwrap();

            (root, data)
        };
        fs::write(root, &data).await.unwrap();
    }
}

use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use db::{commands::EditPiece, Db, Piece, PieceId};
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
                }
            }
        }
        let (root, data) = {
            let db = data.read().unwrap();
            let root = data_file(db.root.clone());
            let data = bincode::serialize::<Db>(&db).unwrap();

            (root, data)
        };
        fs::write(root, &data).await.unwrap();
    }
}

use std::sync::{Arc, RwLock};

use db::{commands::EditPiece, Db};
use tokio::{fs, sync::mpsc};

use super::{data_file, DbBackend};

#[derive(Debug, Clone)]
pub struct DbHandle {
    outgoing: mpsc::UnboundedSender<AppAction>,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    Undo,
    Redo,
    Db(DbAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DbAction {
    EditPiece(EditPiece),
}

pub fn start_db_task(backend: Arc<RwLock<DbBackend>>) -> DbHandle {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(db_actor(rx, backend));

    DbHandle { outgoing: tx }
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

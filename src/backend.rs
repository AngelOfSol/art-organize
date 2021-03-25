use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::Arc,
};

use anyhow::anyhow;
use chrono::Local;
use tokio::fs;

use db::{commands::AttachBlob, Blob, BlobId, BlobType, Db, Piece};

pub mod actor;

use crate::undo::UndoStack;

#[derive(Clone, Debug)]
pub struct DbBackend {
    pub root: PathBuf,
    pub inner: UndoStack<Db>,
}

impl Deref for DbBackend {
    type Target = UndoStack<Db>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for DbBackend {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub fn data_file(mut path: PathBuf) -> PathBuf {
    path.push("data.aodb");
    path
}

impl DbBackend {
    pub async fn save(&self) -> anyhow::Result<()> {
        fs::write(
            data_file(self.root.clone()),
            bincode::serialize::<Db>(self)?,
        )
        .await?;
        Ok(())
    }

    pub async fn from_path(root: PathBuf) -> anyhow::Result<Self> {
        let db = bincode::deserialize::<Db>(&fs::read(data_file(root.clone())).await?)?;
        Ok(Self {
            root,
            inner: UndoStack::new(db),
        })
    }

    pub async fn init_at_path(root: PathBuf) -> anyhow::Result<Self> {
        let db = Db::default();
        let ret = Self {
            root,
            inner: UndoStack::new(db),
        };

        ret.save().await?;

        Ok(ret)
    }

    pub fn storage_for(&self, id: BlobId) -> PathBuf {
        let mut temp = self.root.clone();
        temp.push(self.inner.storage_for(id));
        temp
    }
}

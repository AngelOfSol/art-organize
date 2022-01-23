use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use tokio::fs;

use db::{v2::DbV2 as Db, BlobId, Db as DbV1};

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

    pub async fn from_directory(root: PathBuf) -> anyhow::Result<Self> {
        Self::from_file(data_file(root.clone())).await
    }

    pub async fn from_file(mut root: PathBuf) -> anyhow::Result<Self> {
        let data = &fs::read(root.clone()).await?;
        let db = bincode::deserialize::<DbV1>(data)
            .map(|item| item.into())
            .or_else(|_| bincode::deserialize::<Db>(data))?;
        root.pop();
        Ok(Self {
            root,
            inner: UndoStack::new(db),
        })
    }

    pub async fn init_at_directory(root: PathBuf) -> anyhow::Result<Self> {
        let db = Db::default();
        let ret = Self {
            root,
            inner: UndoStack::new(db),
        };

        Ok(ret)
    }

    pub fn storage_for(&self, id: BlobId) -> PathBuf {
        let mut temp = self.root.clone();
        temp.push(self.inner.storage_for(id));
        temp
    }
}

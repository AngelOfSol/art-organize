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

use db::{commands::AttachBlob, Blob, BlobType, Db, MaybeBlob, Piece};

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

    pub async fn _load_blob(&mut self, file: PathBuf) -> anyhow::Result<MaybeBlob> {
        let file_name = file
            .file_name()
            .and_then(|x| x.to_str())
            .ok_or_else(|| anyhow!("invalid file name: {:?}", file.file_name()))?;

        let data = Arc::new(fs::read(&file).await?);

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let hash = hasher.finish();

        let blob_id = self
            .blobs()
            .find(|(_, blob)| blob.hash == hash)
            .map(|(id, _)| id)
            .filter(|id| self.inner[*id].data == data);

        if let Some(blob_id) = blob_id {
            Ok(MaybeBlob::Id(blob_id))
        } else {
            Ok(MaybeBlob::Value(Blob {
                file_name: file_name.to_string(),
                hash,
                data,
                blob_type: BlobType::Canon,
                added: Local::now(),
            }))
        }
    }

    pub async fn add_file(&mut self, file: PathBuf) -> anyhow::Result<()> {
        let file_name = file
            .file_name()
            .and_then(|x| x.to_str())
            .ok_or_else(|| anyhow!("invalid file name: {:?}", file.file_name()))?;

        let piece = Piece {
            name: file_name.to_owned(),
            ..Piece::default()
        };

        let piece_id = self.create_piece(piece);

        let data = Arc::new(fs::read(&file).await?);

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let hash = hasher.finish();

        let blob_id = self
            .blobs()
            .find(|(_, blob)| blob.hash == hash)
            .map(|(id, _)| id)
            .filter(|id| self.inner[*id].data == data);

        let blob_id = if let Some(blob_id) = blob_id {
            blob_id
        } else {
            // self.blobs.insert(Blob {
            //     file_name: file_name.to_string(),
            //     hash,
            //     data,
            //     blob_type: BlobType::Canon,
            //     added: Local::now(),
            // })
            panic!()
        };

        self.inner.attach_blob(AttachBlob {
            src: piece_id,
            dest: blob_id,
        });

        self.save().await?;

        Ok(())
    }
}

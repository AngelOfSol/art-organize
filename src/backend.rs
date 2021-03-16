use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use anyhow::anyhow;
use chrono::Local;
use tokio::{fs, sync::mpsc};

use db::{Blob, BlobType, Db, MaybeBlob, Piece};

pub struct Backend {
    root: PathBuf,
    db: Db,
}

fn data_file(mut path: PathBuf) -> PathBuf {
    path.push("data.aodb");
    path
}

impl Backend {
    pub async fn save(&self) -> anyhow::Result<()> {
        fs::write(data_file(self.root.clone()), bincode::serialize(&self.db)?).await?;
        Ok(())
    }

    pub async fn from_path(root: PathBuf) -> anyhow::Result<Self> {
        let db = bincode::deserialize(&fs::read(data_file(root.clone())).await?)?;
        Ok(Self { root, db })
    }

    pub async fn init_at_path(root: PathBuf) -> anyhow::Result<Self> {
        let db = Db::default();
        let ret = Self { root, db };

        ret.save().await?;

        Ok(ret)
    }

    pub async fn load_blob(&mut self, file: PathBuf) -> anyhow::Result<MaybeBlob> {
        let file_name = file
            .file_name()
            .and_then(|x| x.to_str())
            .ok_or_else(|| anyhow!("invalid file name: {:?}", file.file_name()))?;

        let data = Arc::new(fs::read(&file).await?);

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let hash = hasher.finish();

        let blob_id = self
            .db
            .blobs
            .iter()
            .find(|(_, blob)| blob.hash == hash)
            .map(|(id, _)| id)
            .filter(|id| self.db.blobs[*id].data == data);

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

        let piece_id = self.db.pieces.insert(piece);

        let data = Arc::new(fs::read(&file).await?);

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let hash = hasher.finish();

        let blob_id = self
            .db
            .blobs
            .iter()
            .find(|(_, blob)| blob.hash == hash)
            .map(|(id, _)| id)
            .filter(|id| self.db.blobs[*id].data == data);

        let blob_id = if let Some(blob_id) = blob_id {
            blob_id
        } else {
            self.db.blobs.insert(Blob {
                file_name: file_name.to_string(),
                hash,
                data,
                blob_type: BlobType::Canon,
                added: Local::now(),
            })
        };

        self.db.media.insert((piece_id, blob_id));

        self.save().await?;

        Ok(())
    }

    pub fn query_pieces(&self) -> impl Iterator<Item = &Piece> {
        self.db.pieces.iter().map(|(_, data)| data)
    }
    pub fn query_blobs(&self) -> impl Iterator<Item = &Blob> {
        self.db.blobs.iter().map(|(_, data)| data)
    }
}

pub use self::piece::{Piece, PieceId};
use super::{
    serialized::{
        blob::{Blob, BlobId, BlobType},
        tag::{Tag, TagId},
        tag_category::{Category, CategoryId},
    },
    DbV1,
};
use crate::table::Table;
use crate::traits::{DeleteFrom, EditFrom, IdExist};
use commands::{AttachBlob, AttachCategory, AttachTag};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

pub mod commands;
pub mod piece;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Clone)]
pub struct DbV2 {
    pub pieces: Table<Piece>,
    pub blobs: Table<Blob>,
    pub tags: Table<Tag>,
    pub categories: Table<Category>,

    pub media: BTreeSet<(PieceId, BlobId)>,
    pub piece_tags: BTreeSet<(PieceId, TagId)>,
    pub tag_category: BTreeMap<TagId, CategoryId>,
}

impl From<DbV1> for DbV2 {
    fn from(value: DbV1) -> Self {
        Self {
            pieces: value
                .pieces
                .iter()
                .map(|(id, value)| {
                    (
                        usize::from(id),
                        Piece {
                            description: value.name.clone(),
                            added: value.added,
                            base_price: value.base_price,
                            tip_price: value.tip_price,
                        },
                    )
                })
                .collect(),
            blobs: value.blobs,
            tags: value.tags,
            categories: value.categories,
            media: value
                .media
                .into_iter()
                .map(|(lhs, rhs)| (usize::from(lhs).into(), rhs))
                .collect(),
            piece_tags: value
                .piece_tags
                .into_iter()
                .map(|(lhs, rhs)| (usize::from(lhs).into(), rhs))
                .collect(),
            tag_category: value.tag_category,
        }
    }
}

impl DbV2 {
    pub fn create_blob(&mut self, data: Blob) -> BlobId {
        self.blobs.insert(data)
    }

    pub fn attach_blob(&mut self, AttachBlob { src, dest }: AttachBlob) -> bool {
        self.media.insert((src, dest))
    }

    pub fn attach_category(&mut self, AttachCategory { src, dest }: AttachCategory) -> bool {
        match dest {
            Some(new_category) => {
                self.tag_category.insert(src, new_category);
                true
            }
            None => self.tag_category.remove(&src).is_some(),
        }
    }
    pub fn attach_tag(&mut self, AttachTag { src, dest }: AttachTag) -> bool {
        self.piece_tags.insert((src, dest))
    }

    pub fn remove_tag(&mut self, AttachTag { src, dest }: AttachTag) -> bool {
        self.piece_tags.remove(&(src, dest))
    }

    pub fn create_piece(&mut self, data: Piece) -> PieceId {
        self.pieces.insert(data)
    }
    pub fn create_tag(&mut self, data: Tag) -> TagId {
        self.tags.insert(data)
    }
    pub fn create_category(&mut self, data: Category) -> CategoryId {
        self.categories.insert(data)
    }

    pub fn blobs_for_piece(&self, piece_id: PieceId) -> impl Iterator<Item = BlobId> + Clone + '_ {
        self.media
            .iter()
            .filter(move |(id, _)| id == &piece_id)
            .map(|(_, id)| *id)
    }
    pub fn pieces_for_blob(&self, blob_id: BlobId) -> impl Iterator<Item = PieceId> + Clone + '_ {
        self.media
            .iter()
            .filter(move |(_, id)| id == &blob_id)
            .map(|(id, _)| *id)
    }

    pub fn primary_blob_for_piece(&self, piece_id: PieceId) -> Option<BlobId> {
        self.blobs_for_piece(piece_id)
            .find(|blob_id| self.blobs[*blob_id].blob_type == BlobType::Canon)
    }

    pub fn tags_for_piece(&self, piece_id: PieceId) -> impl Iterator<Item = TagId> + Clone + '_ {
        self.piece_tags
            .iter()
            .filter(move |(id, _)| id == &piece_id)
            .map(|(_, id)| *id)
    }

    pub fn pieces_for_tag(&self, tag_id: TagId) -> impl Iterator<Item = PieceId> + Clone + '_ {
        self.piece_tags
            .iter()
            .filter(move |(_, id)| id == &tag_id)
            .map(|(id, _)| *id)
    }

    pub fn tags_for_category(
        &self,
        category_id: CategoryId,
    ) -> impl Iterator<Item = TagId> + Clone + '_ {
        self.tag_category
            .iter()
            .filter(move |(_, id)| **id == category_id)
            .map(|(id, _)| *id)
    }

    pub fn category_for_tag(&self, tag_id: TagId) -> Option<CategoryId> {
        self.tag_category.get(&tag_id).copied()
    }

    pub fn pieces(&self) -> impl Iterator<Item = (PieceId, &'_ Piece)> {
        self.pieces.iter()
    }
    pub fn blobs(&self) -> impl Iterator<Item = (BlobId, &'_ Blob)> {
        self.blobs.iter()
    }
    pub fn tags(&self) -> impl Iterator<Item = (TagId, &'_ Tag)> {
        self.tags.iter()
    }
    pub fn categories(&self) -> impl Iterator<Item = (CategoryId, &'_ Category)> {
        self.categories.iter()
    }

    pub fn storage_for(&self, id: BlobId) -> PathBuf {
        self.blobs[id].storage_name(id)
    }

    pub fn exists<Id: IdExist<Self>>(&self, id: Id) -> bool {
        id.exists_in(self)
    }

    pub fn delete<Id: DeleteFrom<Self>>(&mut self, id: Id) -> bool {
        id.delete_from(self)
    }

    pub fn edit<Data: EditFrom<Self>>(&mut self, data: Data) -> bool {
        data.edit_from(self)
    }
}

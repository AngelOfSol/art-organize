#![feature(btree_retain)]

pub use self::{
    blob::{Blob, BlobId, BlobType},
    contained_piece::ContainedPiece,
    media_type::MediaType,
    piece::{Piece, PieceId},
    source_type::SourceType,
    tag::{Tag, TagId},
    tag_category::{Category, CategoryId},
};
use commands::{AttachBlob, AttachCategory, AttachTag};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};
use table::Table;
use traits::{DeleteFrom, EditFrom, IdExist};

mod blob;
pub mod commands;
mod contained_piece;
mod media_type;
mod piece;
mod source_type;
mod table;
mod tag;
mod tag_category;
pub mod traits;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Clone)]
pub struct Db {
    pieces: Table<Piece>,
    blobs: Table<Blob>,
    tags: Table<Tag>,
    categories: Table<Category>,

    media: BTreeSet<(PieceId, BlobId)>,
    piece_tags: BTreeSet<(PieceId, TagId)>,
    tag_category: BTreeMap<TagId, CategoryId>,
}

impl Db {
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

    pub fn blobs_for_piece(&self, piece: PieceId) -> impl Iterator<Item = BlobId> + Clone + '_ {
        self.media
            .iter()
            .filter(move |(id, _)| id == &piece)
            .map(|(_, id)| *id)
    }
    pub fn pieces_for_blob(&self, blob: BlobId) -> impl Iterator<Item = PieceId> + Clone + '_ {
        self.media
            .iter()
            .filter(move |(_, id)| id == &blob)
            .map(|(id, _)| *id)
    }

    pub fn primary_blob_for_piece(&self, piece: PieceId) -> Option<BlobId> {
        self.blobs_for_piece(piece)
            .find(|blob_id| self[blob_id].blob_type == BlobType::Canon)
    }

    pub fn tags_for_piece(&self, piece: PieceId) -> impl Iterator<Item = TagId> + Clone + '_ {
        self.piece_tags
            .iter()
            .filter(move |(id, _)| id == &piece)
            .map(|(_, id)| *id)
    }

    pub fn pieces_for_tag(&self, tag: TagId) -> impl Iterator<Item = PieceId> + Clone + '_ {
        self.piece_tags
            .iter()
            .filter(move |(_, id)| id == &tag)
            .map(|(id, _)| *id)
    }

    pub fn tags_for_category(
        &self,
        category: CategoryId,
    ) -> impl Iterator<Item = TagId> + Clone + '_ {
        self.tag_category
            .iter()
            .filter(move |(_, id)| **id == category)
            .map(|(id, _)| *id)
    }

    pub fn category_for_tag(&self, tag: TagId) -> Option<CategoryId> {
        self.tag_category.get(&tag).copied()
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
        self[id].storage_name(id)
    }

    pub fn exists<Id: IdExist>(&self, id: Id) -> bool {
        id.exists_in(self)
    }

    pub fn delete<Id: DeleteFrom>(&mut self, id: Id) -> bool {
        id.delete_from(self)
    }

    pub fn edit<Data: EditFrom>(&mut self, data: Data) -> bool {
        data.edit_from(self)
    }
}

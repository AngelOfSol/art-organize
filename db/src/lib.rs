pub use self::{
    blob::{Blob, BlobId, BlobType},
    contained_piece::ContainedPiece,
    media_type::MediaType,
    piece::{Piece, PieceId},
    source_type::SourceType,
    tag::Tag,
    tag_category::TagCategory,
};
use self::{tag::TagId, tag_category::TagCategoryId};
use commands::{AttachBlob, EditPiece};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Index,
    path::PathBuf,
};
use table::Table;

mod blob;
pub mod commands;
mod contained_piece;
mod media_type;
mod piece;
mod source_type;
mod table;
mod tag;
mod tag_category;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Clone)]
pub struct Db {
    pieces: Table<Piece>,
    blobs: Table<Blob>,
    tags: Table<Tag>,
    tag_categories: Table<TagCategory>,

    media: BTreeSet<(PieceId, BlobId)>,
    piece_tags: BTreeSet<(PieceId, TagId)>,
    tag_category: BTreeMap<TagId, TagCategoryId>,
}

impl Db {
    pub fn create_blob(&mut self, data: Blob) -> BlobId {
        self.blobs.insert(data)
    }

    pub fn attach_blob(&mut self, AttachBlob { src, dest }: AttachBlob) -> bool {
        self.media.insert((src, dest))
    }

    pub fn create_piece(&mut self, data: Piece) -> PieceId {
        self.pieces.insert(data)
    }
    pub fn edit_piece(&mut self, EditPiece { id, data }: EditPiece) -> bool {
        if let Some(piece) = self.pieces.get_mut(id) {
            *piece = data;
            true
        } else {
            false
        }
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
        category: TagCategoryId,
    ) -> impl Iterator<Item = TagId> + Clone + '_ {
        self.tag_category
            .iter()
            .filter(move |(_, id)| **id == category)
            .map(|(id, _)| *id)
    }

    pub fn category_for_tag(&self, tag: TagId) -> Option<TagCategoryId> {
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
    pub fn tag_categories(&self) -> impl Iterator<Item = (TagCategoryId, &'_ TagCategory)> {
        self.tag_categories.iter()
    }

    pub fn storage_for(&self, id: BlobId) -> PathBuf {
        self[id].storage_name(id)
    }

    pub fn exists<Id: IdExist>(&self, id: Id) -> bool {
        id.exists_in(self)
    }
}

pub trait IdExist {
    fn exists_in(self, db: &Db) -> bool;
}

impl IdExist for BlobId {
    fn exists_in(self, db: &Db) -> bool {
        db.blobs.has(self)
    }
}
impl IdExist for PieceId {
    fn exists_in(self, db: &Db) -> bool {
        db.pieces.has(self)
    }
}
impl IdExist for TagId {
    fn exists_in(self, db: &Db) -> bool {
        db.tags.has(self)
    }
}
impl IdExist for TagCategoryId {
    fn exists_in(self, db: &Db) -> bool {
        db.tag_categories.has(self)
    }
}

impl Index<PieceId> for Db {
    type Output = Piece;

    fn index(&self, index: PieceId) -> &Self::Output {
        &self.pieces[index]
    }
}

impl Index<BlobId> for Db {
    type Output = Blob;

    fn index(&self, index: BlobId) -> &Self::Output {
        &self.blobs[index]
    }
}

impl Index<TagId> for Db {
    type Output = Tag;

    fn index(&self, index: TagId) -> &Self::Output {
        &self.tags[index]
    }
}

impl<'a, T: Copy> Index<&'a T> for Db
where
    Db: Index<T>,
{
    type Output = <Db as Index<T>>::Output;

    fn index(&self, index: &'a T) -> &Self::Output {
        self.index(*index)
    }
}

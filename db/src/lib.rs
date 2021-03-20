pub use self::{
    blob::{Blob, BlobId, BlobType, MaybeBlob},
    contained_piece::ContainedPiece,
    media_type::MediaType,
    piece::{Piece, PieceId},
    source_type::SourceType,
    tag::Tag,
    tag_category::TagCategory,
};
use self::{tag::TagId, tag_category::TagCategoryId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use table::Table;

mod blob;
mod contained_piece;
mod media_type;
mod piece;
mod source_type;
mod table;
mod tag;
mod tag_category;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Clone)]
pub struct Db {
    pub pieces: Table<Piece>,
    pub blobs: Table<Blob>,
    pub tags: Table<Tag>,
    pub tag_categories: Table<TagCategory>,

    pub media: BTreeSet<(PieceId, BlobId)>,
    pub piece_tags: BTreeSet<(PieceId, TagId)>,
    pub tag_category: BTreeMap<TagId, TagCategoryId>,
}

// TODO implement index for all ids
// TODO implement helper methods for db.pieces.iter() etc

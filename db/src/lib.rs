pub use self::serialized::{
    blob::{Blob, BlobId, BlobType},
    media_type::MediaType,
    source_type::SourceType,
    tag::{Tag, TagId},
    tag_category::{Category, CategoryId},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};
use table::Table;
use traits::{DeleteFrom, EditFrom, IdExist};

mod serialized;
mod table;
pub mod traits;
pub mod v2;

pub use v2::DbV2 as Db;
pub use v2::{Piece, PieceId};

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Clone)]
pub struct DbV1 {
    pieces: Table<self::serialized::piece::Piece>,
    blobs: Table<Blob>,
    tags: Table<Tag>,
    categories: Table<Category>,

    media: BTreeSet<(PieceId, BlobId)>,
    piece_tags: BTreeSet<(PieceId, TagId)>,
    tag_category: BTreeMap<TagId, CategoryId>,
}

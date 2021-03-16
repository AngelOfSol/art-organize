use serde::{Deserialize, Serialize};

use crate::{Blob, Piece, Tag, TagCategory};
#[derive(Debug, Serialize, Deserialize)]
pub struct ContainedPiece {
    piece: Piece,
    blobs: Vec<Blob>,
    tags: Vec<(Tag, Option<TagCategory>)>,
}

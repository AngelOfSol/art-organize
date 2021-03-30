use serde::{Deserialize, Serialize};

use crate::{Blob, Category, Piece, Tag};
#[derive(Debug, Serialize, Deserialize)]
pub struct ContainedPiece {
    piece: Piece,
    blobs: Vec<Blob>,
    tags: Vec<(Tag, Option<Category>)>,
}

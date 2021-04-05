use crate::{Blob, BlobId, Category, CategoryId, Piece, PieceId, Tag, TagId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attach<Left, Right> {
    pub src: Left,
    pub dest: Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edit<DataId, Data> {
    pub id: DataId,
    pub data: Data,
}

pub type AttachBlob = Attach<PieceId, BlobId>;
pub type AttachCategory = Attach<TagId, Option<CategoryId>>;
pub type AttachTag = Attach<PieceId, TagId>;

pub type EditPiece = Edit<PieceId, Piece>;
pub type EditBlob = Edit<BlobId, Blob>;
pub type EditTag = Edit<TagId, Tag>;
pub type EditCategory = Edit<CategoryId, Category>;

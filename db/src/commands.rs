use crate::{Blob, BlobId, Piece, PieceId};

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
pub type EditPiece = Edit<PieceId, Piece>;
pub type EditBlob = Edit<BlobId, Blob>;

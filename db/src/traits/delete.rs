use crate::{BlobId, Db, PieceId};

use super::DeleteFrom;

impl DeleteFrom for PieceId {
    fn delete_from(self, db: &mut Db) -> bool {
        if db.exists(self) {
            db.pieces.remove(self);
            db.media.retain(|(piece, _)| *piece != self);
            db.piece_tags.retain(|(piece, _)| *piece != self);

            true
        } else {
            false
        }
    }
}

impl DeleteFrom for BlobId {
    fn delete_from(self, db: &mut Db) -> bool {
        if db.exists(self) {
            db.blobs.remove(self);
            db.media.retain(|(_, blob)| *blob != self);
            true
        } else {
            false
        }
    }
}

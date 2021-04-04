use crate::{tag::TagId, tag_category::CategoryId, BlobId, Db, PieceId};

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

impl DeleteFrom for TagId {
    fn delete_from(self, db: &mut Db) -> bool {
        if db.exists(self) {
            db.tags.remove(self);
            db.piece_tags.retain(|(_, tag)| *tag != self);
            db.tag_category.remove(&self);
            true
        } else {
            false
        }
    }
}

impl DeleteFrom for CategoryId {
    fn delete_from(self, db: &mut Db) -> bool {
        if db.exists(self) {
            db.categories.remove(self);
            db.tag_category.retain(|_, tag| *tag != self);
            true
        } else {
            false
        }
    }
}

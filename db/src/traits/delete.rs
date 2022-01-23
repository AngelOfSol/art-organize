use crate::{v2::DbV2, BlobId, CategoryId, TagId};

use super::DeleteFrom;

impl DeleteFrom<DbV2> for crate::v2::PieceId {
    fn delete_from(self, db: &mut DbV2) -> bool {
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

impl DeleteFrom<DbV2> for BlobId {
    fn delete_from(self, db: &mut DbV2) -> bool {
        if db.exists(self) {
            db.blobs.remove(self);
            db.media.retain(|(_, blob)| *blob != self);
            true
        } else {
            false
        }
    }
}

impl DeleteFrom<DbV2> for TagId {
    fn delete_from(self, db: &mut DbV2) -> bool {
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

impl DeleteFrom<DbV2> for CategoryId {
    fn delete_from(self, db: &mut DbV2) -> bool {
        if db.exists(self) {
            db.categories.remove(self);
            db.tag_category.retain(|_, tag| *tag != self);
            true
        } else {
            false
        }
    }
}

use crate::{v2::DbV2, BlobId, CategoryId, TagId};

use super::IdExist;

impl IdExist<DbV2> for BlobId {
    fn exists_in(self, db: &DbV2) -> bool {
        db.blobs.has(self)
    }
}
impl IdExist<DbV2> for crate::v2::PieceId {
    fn exists_in(self, db: &DbV2) -> bool {
        db.pieces.has(self)
    }
}
impl IdExist<DbV2> for TagId {
    fn exists_in(self, db: &DbV2) -> bool {
        db.tags.has(self)
    }
}
impl IdExist<DbV2> for CategoryId {
    fn exists_in(self, db: &DbV2) -> bool {
        db.categories.has(self)
    }
}

impl<'a, T: Copy> IdExist<DbV2> for &'a T
where
    T: IdExist<DbV2>,
{
    fn exists_in(self, db: &DbV2) -> bool {
        db.exists(*self)
    }
}

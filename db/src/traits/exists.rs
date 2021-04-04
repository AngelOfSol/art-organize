use crate::{tag::TagId, tag_category::CategoryId, BlobId, Db, PieceId};

use super::IdExist;

impl IdExist for BlobId {
    fn exists_in(self, db: &Db) -> bool {
        db.blobs.has(self)
    }
}
impl IdExist for PieceId {
    fn exists_in(self, db: &Db) -> bool {
        db.pieces.has(self)
    }
}
impl IdExist for TagId {
    fn exists_in(self, db: &Db) -> bool {
        db.tags.has(self)
    }
}
impl IdExist for CategoryId {
    fn exists_in(self, db: &Db) -> bool {
        db.categories.has(self)
    }
}

impl<'a, T: Copy> IdExist for &'a T
where
    T: IdExist,
{
    fn exists_in(self, db: &Db) -> bool {
        db.exists(*self)
    }
}

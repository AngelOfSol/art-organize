use crate::{BlobId, CategoryId, Db, PieceId, TagId};

use super::IdExist;

impl IdExist<Db> for BlobId {
    fn exists_in(self, db: &Db) -> bool {
        db.blobs.has(self)
    }
}
impl IdExist<Db> for PieceId {
    fn exists_in(self, db: &Db) -> bool {
        db.pieces.has(self)
    }
}
impl IdExist<Db> for TagId {
    fn exists_in(self, db: &Db) -> bool {
        db.tags.has(self)
    }
}
impl IdExist<Db> for CategoryId {
    fn exists_in(self, db: &Db) -> bool {
        db.categories.has(self)
    }
}

impl<'a, T: Copy> IdExist<Db> for &'a T
where
    T: IdExist<Db>,
{
    fn exists_in(self, db: &Db) -> bool {
        db.exists(*self)
    }
}

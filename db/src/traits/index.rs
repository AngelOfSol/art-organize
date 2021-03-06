use std::ops::Index;

use crate::{v2::Piece, v2::PieceId, Blob, BlobId, Category, CategoryId, Db, Tag, TagId};

impl Index<PieceId> for Db {
    type Output = Piece;

    fn index(&self, index: PieceId) -> &Self::Output {
        &self.pieces[index]
    }
}

impl Index<BlobId> for Db {
    type Output = Blob;

    fn index(&self, index: BlobId) -> &Self::Output {
        &self.blobs[index]
    }
}

impl Index<TagId> for Db {
    type Output = Tag;

    fn index(&self, index: TagId) -> &Self::Output {
        &self.tags[index]
    }
}

impl Index<CategoryId> for Db {
    type Output = Category;

    fn index(&self, index: CategoryId) -> &Self::Output {
        &self.categories[index]
    }
}

impl<'a, T: Copy> Index<&'a T> for Db
where
    Db: Index<T>,
{
    type Output = <Db as Index<T>>::Output;

    fn index(&self, index: &'a T) -> &Self::Output {
        self.index(*index)
    }
}

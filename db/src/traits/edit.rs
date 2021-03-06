use crate::{
    v2::commands::{EditBlob, EditCategory, EditPiece, EditTag},
    Db,
};

use super::EditFrom;

impl EditFrom<Db> for EditPiece {
    fn edit_from(self, db: &mut Db) -> bool {
        if let Some(piece) = db.pieces.get_mut(self.id) {
            *piece = self.data;
            true
        } else {
            false
        }
    }
}
impl EditFrom<Db> for EditBlob {
    fn edit_from(self, db: &mut Db) -> bool {
        if let Some(blob) = db.blobs.get_mut(self.id) {
            *blob = self.data;
            true
        } else {
            false
        }
    }
}

impl EditFrom<Db> for EditTag {
    fn edit_from(self, db: &mut Db) -> bool {
        if let Some(tag) = db.tags.get_mut(self.id) {
            *tag = self.data;
            true
        } else {
            false
        }
    }
}

impl EditFrom<Db> for EditCategory {
    fn edit_from(self, db: &mut Db) -> bool {
        if let Some(category) = db.categories.get_mut(self.id) {
            *category = self.data;
            true
        } else {
            false
        }
    }
}

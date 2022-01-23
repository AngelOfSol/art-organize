use db::{Db, Piece, PieceId};

use crate::model::{Condition, DateOp, Search};

impl Search {
    pub fn execute<'a>(&'a self, db: &'a Db) -> impl Iterator<Item = PieceId> + 'a {
        db.pieces()
            .filter(move |item| self.evaluate_internal(item, db).unwrap_or(true))
            .map(|(id, _)| id)
    }
    fn evaluate_internal(&self, value: &(PieceId, &Piece), db: &Db) -> Option<bool> {
        match self {
            Search::Or(inner) => Some(
                inner
                    .iter()
                    .any(|item| item.evaluate_internal(value, db).unwrap_or(true)),
            ),
            Search::And(inner) => Some(
                inner
                    .iter()
                    .all(|item| item.evaluate_internal(value, db).unwrap_or(true)),
            ),
            Search::Negate(inner) => inner.evaluate_internal(value, db).map(|item| !item),

            Search::Test(test) => evaluate_test(test, value, db),
        }
    }
}

/// Returns Some(bool) evaluating the condition, returning None
/// if the condition doesn't make sense (non-existent category, or tag for example)
fn evaluate_test(test: &Condition, (id, piece): &(PieceId, &Piece), db: &Db) -> Option<bool> {
    match test {
        Condition::Tag(tag_name) => {
            let (searched, _) = db.tags().find(|(_, tag)| &tag.name == tag_name)?;
            // either the piece contains the tag
            // OR
            // no tags of this name exist in the database
            Some(db.tags_for_piece(*id).any(|tag| tag == searched))
        }
        Condition::TagWithCategory(category_name, tag_name) => {
            let (searched, _) =
                db.tags()
                    .filter(|(_, tag)| &tag.name == tag_name)
                    .find(|(searched, _)| {
                        db.category_for_tag(*searched)
                            .map(|category_id| &db[category_id].name)
                            == category_name.as_ref()
                    })?;
            // either the piece contains the category:tag
            // OR
            // no category:tag of this name exist in the database
            Some(db.tags_for_piece(*id).any(|tag| tag == searched))
        }
        Condition::DateAdded(op, date) => Some(match op {
            DateOp::Before => &piece.added <= date,
            DateOp::After => &piece.added >= date,
        }),
    }
}

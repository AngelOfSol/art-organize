use crate::Db;

mod delete;
mod edit;
mod exists;
mod index;

pub trait EditFrom<Db> {
    fn edit_from(self, db: &mut Db) -> bool;
}
pub trait DeleteFrom<Db> {
    fn delete_from(self, db: &mut Db) -> bool;
}

pub trait IdExist<Db> {
    fn exists_in(self, db: &Db) -> bool;
}

use crate::Db;

mod delete;
mod edit;
mod exists;
mod index;

pub trait EditFrom {
    fn edit_from(self, db: &mut Db) -> bool;
}
pub trait DeleteFrom {
    fn delete_from(self, db: &mut Db) -> bool;
}

pub trait IdExist {
    fn exists_in(self, db: &Db) -> bool;
}

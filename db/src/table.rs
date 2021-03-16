use serde::{Deserialize, Serialize};
use slab::Slab;
use std::{collections::BTreeMap, marker::PhantomData, ops::Index};
use undo::{Command, Merge};

#[derive(Debug, Serialize, Deserialize)]
pub struct TableId<T>(usize, PhantomData<T>);

pub enum MaybeEntry<T> {
    Id(TableId<T>),
    Value(T),
}

impl<T> Clone for TableId<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}
impl<T> Copy for TableId<T> {}

impl<T> From<usize> for TableId<T> {
    fn from(value: usize) -> Self {
        TableId(value, PhantomData)
    }
}

impl<T> PartialEq for TableId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for TableId<T> {}

impl<T> PartialOrd for TableId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for TableId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

pub enum TableCommand<T, TCommand> {
    Added(T, Option<TableId<T>>),
    Removed(Option<T>, TableId<T>),
    Changed(TableId<T>, TCommand),
}

impl<T, TCommand> TableCommand<T, TCommand> {
    pub fn add(value: T) -> Self {
        Self::Added(value, None)
    }
    pub fn remove(id: TableId<T>) -> Self {
        Self::Removed(None, id)
    }

    pub fn edit(id: TableId<T>, inner: TCommand) -> Self {
        Self::Changed(id, inner)
    }
}

impl<T: Clone, TCommand: Command<Target = T, Error = ()>> Command for TableCommand<T, TCommand> {
    type Target = Table<T>;

    type Error = ();

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match self {
            TableCommand::Added(item, id) => {
                *id = Some(target.insert(item.clone()));
                Ok(())
            }
            TableCommand::Removed(data, id) => {
                *data = Some(target.remove(*id));
                Ok(())
            }
            TableCommand::Changed(id, command) => command.apply(target.get_mut(*id).unwrap()),
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match self {
            TableCommand::Added(item, id) => {
                *item = target.remove(id.unwrap());
                Ok(())
            }
            TableCommand::Removed(item, id) => {
                *id = target.insert(item.clone().unwrap());
                Ok(())
            }
            TableCommand::Changed(id, command) => command.undo(target.get_mut(*id).unwrap()),
        }
    }

    fn merge(&mut self, command: Self) -> Merge<Self> {
        match (self, command) {
            (TableCommand::Changed(lhs, inner), TableCommand::Changed(rhs, new)) if *lhs == rhs => {
                match inner.merge(new) {
                    Merge::No(command) => Merge::No(TableCommand::Changed(rhs, command)),
                    Merge::Yes => Merge::Yes,
                    Merge::Annul => Merge::Annul,
                }
            }
            (_, command) => Merge::No(command),
        }
    }
}

#[derive(Debug)]
pub struct Table<T> {
    data: Slab<T>,
}
impl<T: PartialEq> PartialEq for Table<T> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().zip(other.iter()).all(|(lhs, rhs)| lhs == rhs)
    }
}
impl<T: Eq> Eq for Table<T> {}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self { data: Slab::new() }
    }
}

impl<T> Index<TableId<T>> for Table<T> {
    type Output = T;

    fn index(&self, index: TableId<T>) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> Table<T> {
    pub fn check<F: Fn(&T, &T) -> bool>(&self, data: T, f: F) -> MaybeEntry<T> {
        if let Some((id, _)) = self.data.iter().find(|(_, item)| f(*item, &data)) {
            MaybeEntry::Id(id.into())
        } else {
            MaybeEntry::Value(data)
        }
    }

    pub fn has(&self, index: TableId<T>) -> bool {
        self.data.contains(index.0)
    }

    pub fn insert(&mut self, data: T) -> TableId<T> {
        self.data.insert(data).into()
    }
    pub fn remove(&mut self, data: TableId<T>) -> T {
        self.data.remove(data.0)
    }

    pub fn get(&self, index: TableId<T>) -> Option<&T> {
        self.data.get(index.0)
    }
    pub fn get_mut(&mut self, index: TableId<T>) -> Option<&mut T> {
        self.data.get_mut(index.0)
    }

    pub fn iter(&self) -> impl Iterator<Item = (TableId<T>, &T)> {
        self.data.iter().map(|(id, value)| (id.into(), value))
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (TableId<T>, &mut T)> {
        self.data.iter_mut().map(|(id, value)| (id.into(), value))
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Table<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = BTreeMap::<usize, T>::deserialize(deserializer)?;
        let table = Table {
            data: data.into_iter().collect(),
        };

        Ok(table)
    }
}

impl<T: Serialize> Serialize for Table<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data: BTreeMap<_, _> = self.data.iter().collect();

        data.serialize(serializer)
    }
}

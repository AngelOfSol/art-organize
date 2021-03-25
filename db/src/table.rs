use serde::{Deserialize, Serialize};
use slab::Slab;
use std::{collections::BTreeMap, fmt::Display, marker::PhantomData, ops::Index};

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

impl<T> Display for TableId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
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

#[allow(dead_code)]
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

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.data.iter().map(|(_, value)| value)
    }
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().map(|(_, value)| value)
    }

    pub fn keys(&self) -> impl Iterator<Item = TableId<T>> + '_ {
        self.data.iter().map(|(id, _)| id.into())
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

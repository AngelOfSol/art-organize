use std::{collections::HashMap, hash::Hash, marker::PhantomData};

use glam::{BVec2, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayoutIds {
    SearchBar,
    MenuBar,
    Tags,
    Main,
}

#[derive(Debug)]
pub enum Dimension {
    Pixels(f32),
    Flex(f32),
}

pub type Row<ItemId> = Group<Item<ItemId>, RowType>;
pub type Column<ItemId> = Group<Item<ItemId>, ColumnType>;

#[derive(Default, Debug)]
pub struct RowType;
#[derive(Default, Debug)]
pub struct ColumnType;

#[derive(Debug)]
pub struct Group<Id, T> {
    data: Vec<(Id, Dimension)>,
    marker: PhantomData<T>,
}

impl<Id, T> Default for Group<Id, T> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub enum Item<ItemId = LayoutIds> {
    Id(ItemId),
    Row(Row<ItemId>),
    Column(Column<ItemId>),
}

impl<ItemId> From<ItemId> for Item<ItemId> {
    fn from(value: ItemId) -> Self {
        Self::Id(value)
    }
}
impl<ItemId> From<Row<ItemId>> for Item<ItemId> {
    fn from(value: Row<ItemId>) -> Self {
        Self::Row(value)
    }
}
impl<ItemId> From<Column<ItemId>> for Item<ItemId> {
    fn from(value: Column<ItemId>) -> Self {
        Self::Column(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LayoutRectangle {
    pub position: Vec2,
    pub size: Vec2,
}

impl<ItemId: Hash + Eq + Copy> Row<ItemId> {
    pub fn layout(
        &self,
        allocated: LayoutRectangle,
        layout: &mut HashMap<ItemId, LayoutRectangle>,
    ) {
        let lengths = self.compute(allocated.size.x);

        let mut position = allocated.position;

        for ((item, _), length) in self.data.iter().zip(lengths) {
            // select x for length

            let allocated = LayoutRectangle {
                position,
                size: Vec2::select(BVec2::new(true, false), Vec2::splat(length), allocated.size),
            };

            position.x += length;

            match item {
                Item::Id(item) => {
                    layout.insert(*item, allocated);
                }
                Item::Row(row) => {
                    row.layout(allocated, layout);
                }
                Item::Column(column) => {
                    column.layout(allocated, layout);
                }
            }
            //
        }
    }
}

impl<ItemId: Hash + Eq + Copy> Column<ItemId> {
    pub fn layout(
        &self,
        allocated: LayoutRectangle,
        layout: &mut HashMap<ItemId, LayoutRectangle>,
    ) {
        let lengths = self.compute(allocated.size.y);

        let mut position = allocated.position;

        for ((item, _), height) in self.data.iter().zip(lengths) {
            // select y for height

            let allocated = LayoutRectangle {
                position,
                size: Vec2::select(BVec2::new(false, true), Vec2::splat(height), allocated.size),
            };

            position.y += height;

            match item {
                Item::Id(item) => {
                    layout.insert(*item, allocated);
                }
                Item::Row(row) => {
                    row.layout(allocated, layout);
                }
                Item::Column(column) => {
                    column.layout(allocated, layout);
                }
            }
            //
        }
    }
}

impl<Id, T> Group<Id, T> {
    pub fn push<I: Into<Id>>(mut self, item: I, dim: Dimension) -> Self {
        self.data.push((item.into(), dim));
        self
    }

    fn compute(&self, scalar: f32) -> impl Iterator<Item = f32> + '_ {
        //
        let total_flex = self.data.iter().fold(0.0, |acc, (_, dim)| match dim {
            Dimension::Pixels(_) => acc,
            Dimension::Flex(units) => acc + *units,
        });
        let total_pixels = self.data.iter().fold(0.0, |acc, (_, dim)| match dim {
            Dimension::Pixels(units) => acc + *units,
            Dimension::Flex(_) => acc,
        });
        let available_for_flex = scalar - total_pixels;

        if available_for_flex < 1.0 && total_flex > 0.0 {
            panic!(
                "expected to have at least 1 pixel to allocate to flex items, instead found {}",
                available_for_flex
            );
        }

        self.data.iter().map(move |(_, dim)| match dim {
            Dimension::Pixels(value) => *value,
            Dimension::Flex(flex) => flex / total_flex * available_for_flex,
        })
    }
}

#[test]
pub fn test_layout() {
    let layout = Column::default()
        .push(LayoutIds::MenuBar, Dimension::Pixels(20.0))
        .push(LayoutIds::SearchBar, Dimension::Pixels(100.0))
        .push(
            Row::default()
                .push(LayoutIds::Tags, Dimension::Pixels(240.0))
                .push(LayoutIds::Main, Dimension::Flex(1.0)),
            Dimension::Flex(1.0),
        );

    let mut data = HashMap::new();

    layout.layout(
        LayoutRectangle {
            position: Vec2::ZERO,
            size: Vec2::new(1280.0, 720.0),
        },
        &mut data,
    );

    assert_eq!(
        data[&LayoutIds::MenuBar],
        LayoutRectangle {
            position: Vec2::new(0.0, 0.0),
            size: Vec2::new(1280.0, 20.0),
        }
    );
    assert_eq!(
        data[&LayoutIds::SearchBar],
        LayoutRectangle {
            position: Vec2::new(0.0, 20.0),
            size: Vec2::new(1280.0, 100.0),
        }
    );
    assert_eq!(
        data[&LayoutIds::Tags],
        LayoutRectangle {
            position: Vec2::new(0.0, 120.0),
            size: Vec2::new(240.0, 600.0),
        }
    );
    assert_eq!(
        data[&LayoutIds::Main],
        LayoutRectangle {
            position: Vec2::new(240.0, 120.0),
            size: Vec2::new(1040.0, 600.0),
        }
    );
}

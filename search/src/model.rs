use chrono::NaiveDate;
use db::{MediaType, SourceType};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Search {
    Or(Vec<Search>),
    And(Vec<Search>),
    Negate(Box<Search>),
    Test(Condition),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Condition {
    Tag(String),
    TagWithCategory(Option<String>, String),
    Source(SourceType),
    Media(MediaType),
    DateAdded(DateOp, NaiveDate),
    Price(PriceType, PriceOp, i64),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateOp {
    Before,
    After,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriceOp {
    GreaterEqual,
    LesserEqual,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriceType {
    Total,
    Tip,
    Base,
}

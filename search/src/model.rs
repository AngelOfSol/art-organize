use chrono::NaiveDate;
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
    DateAdded(DateOp, NaiveDate),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateOp {
    Before,
    After,
}

use crate::model::{Condition, DateOp, PriceOp, PriceType, Search};
use chrono::NaiveDate;
use db::{MediaType, SourceType};
use nom::{
    branch::alt,
    bytes::complete::{self, tag},
    character::complete::space1,
    combinator::{map, map_opt, verify},
    multi::separated_list1,
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};

mod tag;

fn parse_and(input: &str) -> IResult<&str, Search> {
    map(
        verify(
            separated_list1(
                space1,
                alt((parse_or, parse_paren, parse_negate, parse_test)),
            ),
            |inner: &Vec<_>| inner.len() >= 2,
        ),
        Search::And,
    )(input)
}

fn parse_or(input: &str) -> IResult<&str, Search> {
    map(
        verify(
            separated_list1(tag("|"), alt((parse_paren, parse_negate, parse_test))),
            |inner: &Vec<_>| inner.len() >= 2,
        ),
        Search::Or,
    )(input)
}

fn parse_search(input: &str) -> IResult<&str, Search> {
    alt((parse_and, parse_or, parse_paren, parse_negate, parse_test))(input)
}

fn parse_paren(input: &str) -> IResult<&str, Search> {
    terminated(preceded(tag("("), parse_search), tag(")"))(input)
}

fn parse_negate(input: &str) -> IResult<&str, Search> {
    map(
        preceded(tag("!"), alt((parse_paren, parse_negate, parse_test))),
        |value| Search::Negate(Box::new(value)),
    )(input)
}

fn parse_test(input: &str) -> IResult<&str, Search> {
    map(parse_condition, Search::Test)(input)
}

fn parse_whole(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(parse_item, complete::tag(":"), parse_item)(input)
}

fn parse_item(input: &str) -> IResult<&str, &str> {
    complete::take_while(|c: char| {
        !c.is_whitespace() && !matches!(c, ':' | '(' | ')' | '!' | '|' | '>' | '=' | '<')
    })(input)
}

fn parse_condition(input: &str) -> IResult<&str, Condition> {
    alt((
        parse_source,
        parse_media,
        parse_added,
        parse_price,
        parse_tag_with_category,
        parse_tag,
    ))(input)
}

fn parse_source(input: &str) -> IResult<&str, Condition> {
    map_opt(
        verify(parse_whole, |(lhs, _)| *lhs == "source"),
        |(_, rhs)| {
            Some(match rhs {
                "fan" => Condition::Source(SourceType::FanCreation),
                "commission" => Condition::Source(SourceType::Commission),
                "official" => Condition::Source(SourceType::Official),
                _ => return None,
            })
        },
    )(input)
}
fn parse_media(input: &str) -> IResult<&str, Condition> {
    map_opt(
        verify(parse_whole, |(lhs, _)| *lhs == "media"),
        |(_, rhs)| {
            Some(match rhs {
                "image" => Condition::Media(MediaType::Image),
                "text" => Condition::Media(MediaType::Text),
                _ => return None,
            })
        },
    )(input)
}

fn parse_added(input: &str) -> IResult<&str, Condition> {
    map_opt(parse_whole, |(lhs, rhs)| {
        Some(Condition::DateAdded(
            match lhs {
                "after" => DateOp::After,
                "before" => DateOp::Before,
                _ => return None,
            },
            NaiveDate::parse_from_str(rhs, "%m/%d/%Y").ok()?,
        ))
    })(input)
}
fn parse_price(input: &str) -> IResult<&str, Condition> {
    map_opt(
        tuple((parse_item, alt((tag("<="), tag(">="))), parse_item)),
        |(price_type, operation, value)| {
            let price_type = match price_type {
                "total" => PriceType::Total,
                "base" => PriceType::Base,
                "tip" => PriceType::Tip,
                _ => return None,
            };
            let operation = match operation {
                ">=" => PriceOp::GreaterEqual,
                "<=" => PriceOp::LesserEqual,
                _ => unreachable!(),
            };
            let value = value.parse().ok()?;
            Some(Condition::Price(price_type, operation, value))
        },
    )(input)
}

fn parse_tag(input: &str) -> IResult<&str, Condition> {
    map(verify(parse_item, |x: &str| !x.is_empty()), |value| {
        Condition::Tag(value.to_string())
    })(input)
}
fn parse_tag_with_category(input: &str) -> IResult<&str, Condition> {
    map(parse_whole, |(category, tag)| {
        Condition::TagWithCategory(
            (!category.trim().is_empty()).then(|| category.to_string()),
            tag.to_string(),
        )
    })(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::*;
    use Condition::*;
    use Search::*;

    #[test]
    fn test_tag() {
        assert_eq!(parse_tag("input"), Ok(("", Tag("input".to_owned()))));
        assert!(parse_tag("").is_err())
    }
    #[test]
    fn test_tag_with_category() {
        assert_eq!(
            parse_tag_with_category(":input"),
            Ok(("", TagWithCategory(None, "input".to_owned())))
        );
        assert_eq!(
            parse_tag_with_category("category:input"),
            Ok((
                "",
                TagWithCategory(Some("category".to_owned()), "input".to_owned())
            ))
        );
        assert!(parse_tag_with_category("input").is_err())
    }

    #[test]
    fn test_price() {
        assert_eq!(
            parse_price("base>=20"),
            Ok(("", Price(PriceType::Base, PriceOp::GreaterEqual, 20)))
        );
        assert_eq!(
            parse_price("base<=20"),
            Ok(("", Price(PriceType::Base, PriceOp::LesserEqual, 20)))
        );
        assert_eq!(
            parse_price("tip<=20"),
            Ok(("", Price(PriceType::Tip, PriceOp::LesserEqual, 20)))
        );
        assert_eq!(
            parse_price("total<=20"),
            Ok(("", Price(PriceType::Total, PriceOp::LesserEqual, 20)))
        );
        assert!(parse_price("awa>=20").is_err());
        assert!(parse_price("total20").is_err());
        assert!(parse_price("total<=").is_err());
    }

    #[test]
    fn test_added() {
        assert_eq!(
            parse_added("after:10/25/2011"),
            Ok((
                "",
                DateAdded(DateOp::After, NaiveDate::from_ymd(2011, 10, 25))
            ))
        );
        assert_eq!(
            parse_added("before:10/25/2011"),
            Ok((
                "",
                DateAdded(DateOp::Before, NaiveDate::from_ymd(2011, 10, 25))
            ))
        );

        assert!(parse_added("wafter:10/25/2011").is_err());
        assert!(parse_added("after:100/24/2011").is_err());
    }

    #[test]
    fn test_media() {
        assert_eq!(
            parse_media("media:image"),
            Ok(("", Media(MediaType::Image)))
        );
        assert_eq!(parse_media("media:text"), Ok(("", Media(MediaType::Text))));

        assert!(parse_media("media:logs").is_err());
        assert!(parse_media("wmedia:text").is_err());
    }
    #[test]
    fn test_source() {
        assert_eq!(
            parse_source("source:fan"),
            Ok(("", Source(SourceType::FanCreation)))
        );
        assert_eq!(
            parse_source("source:official"),
            Ok(("", Source(SourceType::Official)))
        );
        assert_eq!(
            parse_source("source:commission"),
            Ok(("", Source(SourceType::Commission)))
        );

        assert!(parse_source("source:logs").is_err());
        assert!(parse_source("wsource:commission").is_err());
    }

    #[test]
    fn test_condition() {
        assert_eq!(
            parse_condition("base>=20"),
            Ok(("", Price(PriceType::Base, PriceOp::GreaterEqual, 20)))
        );
        assert_eq!(
            parse_condition("after:10/25/2011"),
            Ok((
                "",
                DateAdded(DateOp::After, NaiveDate::from_ymd(2011, 10, 25))
            ))
        );
        assert_eq!(
            parse_condition("media:image"),
            Ok(("", Media(MediaType::Image)))
        );
        assert_eq!(
            parse_condition("source:fan"),
            Ok(("", Source(SourceType::FanCreation)))
        );
        assert_eq!(parse_condition("input"), Ok(("", Tag("input".to_owned()))));
        assert_eq!(
            parse_condition(":input"),
            Ok(("", TagWithCategory(None, "input".to_owned())))
        );
        assert_eq!(
            parse_condition("category:input"),
            Ok((
                "",
                TagWithCategory(Some("category".to_owned()), "input".to_owned())
            ))
        );

        assert!(parse_condition("(tag)").is_err());
        assert!(parse_condition("!tag").is_err());
    }

    #[test]
    fn test_parse_test() {
        assert_eq!(
            parse_test("base>=20"),
            Ok(("", Test(Price(PriceType::Base, PriceOp::GreaterEqual, 20))))
        );
        assert_eq!(
            parse_search("after:10/25/2011"),
            Ok((
                "",
                Test(DateAdded(DateOp::After, NaiveDate::from_ymd(2011, 10, 25)))
            ))
        );
        assert_eq!(
            parse_test("media:image"),
            Ok(("", Test(Media(MediaType::Image))))
        );
        assert_eq!(
            parse_test("source:fan"),
            Ok(("", Test(Source(SourceType::FanCreation))))
        );
        assert_eq!(parse_test("input"), Ok(("", Test(Tag("input".to_owned())))));
        assert_eq!(
            parse_test(":input"),
            Ok(("", Test(TagWithCategory(None, "input".to_owned()))))
        );
        assert_eq!(
            parse_test("category:input"),
            Ok((
                "",
                Test(TagWithCategory(
                    Some("category".to_owned()),
                    "input".to_owned()
                ))
            ))
        );

        assert!(parse_condition("(tag)").is_err());
        assert!(parse_condition("!tag").is_err());
    }

    #[test]
    fn test_negate() {
        assert_eq!(
            parse_negate("!base>=20"),
            Ok((
                "",
                Negate(Box::new(Test(Price(
                    PriceType::Base,
                    PriceOp::GreaterEqual,
                    20
                ))))
            ))
        );
        assert_eq!(
            parse_negate("!!base>=20"),
            Ok((
                "",
                Negate(Box::new(Negate(Box::new(Test(Price(
                    PriceType::Base,
                    PriceOp::GreaterEqual,
                    20
                ))))))
            ))
        );
    }
    #[test]
    fn test_parens() {
        assert_eq!(
            parse_paren("(base>=20)"),
            Ok(("", Test(Price(PriceType::Base, PriceOp::GreaterEqual, 20))))
        );
    }
    #[test]
    fn test_and() {
        assert_eq!(
            parse_and("base>=20 base<=30"),
            Ok((
                "",
                And(vec![
                    Test(Price(PriceType::Base, PriceOp::GreaterEqual, 20)),
                    Test(Price(PriceType::Base, PriceOp::LesserEqual, 30)),
                ])
            ))
        );
    }
    #[test]
    fn test_or() {
        assert_eq!(
            parse_or("base<=20|base>=30"),
            Ok((
                "",
                Or(vec![
                    Test(Price(PriceType::Base, PriceOp::LesserEqual, 20)),
                    Test(Price(PriceType::Base, PriceOp::GreaterEqual, 30)),
                ])
            ))
        );
    }

    #[test]
    fn test_search() {
        assert_eq!(
            parse_search("!base>=20"),
            Ok((
                "",
                Negate(Box::new(Test(Price(
                    PriceType::Base,
                    PriceOp::GreaterEqual,
                    20
                ))))
            ))
        );

        assert_eq!(
            parse_search("(base>=20)"),
            Ok(("", Test(Price(PriceType::Base, PriceOp::GreaterEqual, 20))))
        );

        assert_eq!(
            parse_search("base>=20 base<=30"),
            Ok((
                "",
                And(vec![
                    Test(Price(PriceType::Base, PriceOp::GreaterEqual, 20)),
                    Test(Price(PriceType::Base, PriceOp::LesserEqual, 30)),
                ])
            ))
        );

        assert_eq!(
            parse_search("base<=20|base>=30"),
            Ok((
                "",
                Or(vec![
                    Test(Price(PriceType::Base, PriceOp::LesserEqual, 20)),
                    Test(Price(PriceType::Base, PriceOp::GreaterEqual, 30)),
                ])
            ))
        );
    }

    #[test]
    fn test_and_or_precedence() {
        assert_eq!(
            parse_search("yumi_lovelace casual_outfit|teaching_outfit"),
            Ok((
                "",
                And(vec![
                    Test(Tag("yumi_lovelace".to_owned())),
                    Or(vec![
                        Test(Tag("casual_outfit".to_owned())),
                        Test(Tag("teaching_outfit".to_owned())),
                    ])
                ])
            ))
        );
        assert_eq!(
            parse_search("casual_outfit|teaching_outfit yumi_lovelace"),
            Ok((
                "",
                And(vec![
                    Or(vec![
                        Test(Tag("casual_outfit".to_owned())),
                        Test(Tag("teaching_outfit".to_owned())),
                    ]),
                    Test(Tag("yumi_lovelace".to_owned())),
                ])
            ))
        );

        assert_eq!(
            parse_search("(yumi_lovelace casual_outfit)|teaching_outfit"),
            Ok((
                "",
                Or(vec![
                    And(vec![
                        Test(Tag("yumi_lovelace".to_owned())),
                        Test(Tag("casual_outfit".to_owned())),
                    ]),
                    Test(Tag("teaching_outfit".to_owned())),
                ])
            ))
        );
    }

    #[test]
    fn test_negate_grouped() {
        assert_eq!(
            parse_search("yumi_lovelace !casual_outfit"),
            Ok((
                "",
                And(vec![
                    Test(Tag("yumi_lovelace".to_owned())),
                    Negate(Box::new(Test(Tag("casual_outfit".to_owned())),))
                ])
            ))
        );
        assert_eq!(
            parse_search("yumi_lovelace|!casual_outfit"),
            Ok((
                "",
                Or(vec![
                    Test(Tag("yumi_lovelace".to_owned())),
                    Negate(Box::new(Test(Tag("casual_outfit".to_owned())),))
                ])
            ))
        );
        assert_eq!(
            parse_search("!yumi_lovelace|!casual_outfit"),
            Ok((
                "",
                Or(vec![
                    Negate(Box::new(Test(Tag("yumi_lovelace".to_owned())))),
                    Negate(Box::new(Test(Tag("casual_outfit".to_owned())),))
                ])
            ))
        );
        assert_eq!(
            parse_search("!(yumi_lovelace|casual_outfit)"),
            Ok((
                "",
                Negate(Box::new(Or(vec![
                    Search::Test(Tag("yumi_lovelace".to_owned())),
                    Test(Tag("casual_outfit".to_owned()))
                ])))
            ))
        );
    }

    #[test]
    fn test_complex() {
        assert_eq!(
            parse_search(
                "(yumi_lovelace !(casual_outfit|teaching_outfit))|(miu_yarai work_outfit)"
            ),
            Ok((
                "",
                Or(vec![
                    And(vec![
                        Test(Tag("yumi_lovelace".to_owned())),
                        Negate(Box::new(Or(vec![
                            Test(Tag("casual_outfit".to_owned())),
                            Test(Tag("teaching_outfit".to_owned()))
                        ])))
                    ]),
                    And(vec![
                        Test(Tag("miu_yarai".to_owned())),
                        Test(Tag("work_outfit".to_owned()))
                    ])
                ])
            ))
        );
    }
}

use std::convert::TryFrom;

#[derive(Debug)]
pub struct Links<'a> {
    pub first: Option<&'a str>,
    pub prev: Option<&'a str>,
    pub next: Option<&'a str>,
    pub last: Option<&'a str>,
}

impl<'a> Default for Links<'a> {
    fn default() -> Self {
        Links {
            first: None,
            prev: None,
            next: None,
            last: None,
        }
    }
}

impl<'a> TryFrom<&'a str> for Links<'a> {
    type Error = String;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let parser_res = parser::links(&value);
        let links = match parser_res {
            Ok((reminder, ref links)) if reminder.is_empty() => Ok(links),
            Ok((ref reminder, _)) => Err(format!(
                "Link header could not be fully parsed: '{}'",
                reminder
            )),
            Err(err) => Err(format!("Link header could not be parsed because {:?}", err)),
        }?;
        let mut res = Links::default();
        for l in links {
            match l.dir {
                parser::Direction::First => {
                    res = Links {
                        first: Some(l.url),
                        ..res
                    }
                }
                parser::Direction::Prev => {
                    res = Links {
                        prev: Some(l.url),
                        ..res
                    }
                }
                parser::Direction::Next => {
                    res = Links {
                        next: Some(l.url),
                        ..res
                    }
                }
                parser::Direction::Last => {
                    res = Links {
                        last: Some(l.url),
                        ..res
                    }
                }
            }
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use spectral::prelude::*;

    #[test]
    fn parse_link_header_value() {
        let value = r#"<https://api.github.com/search/code?q=addClass+user%3Amozilla&page=15>; rel="next", <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=34>; rel="last", <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=1>; rel="first", <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=13>; rel="prev""#;
        let res = Links::try_from(value);

        asserting("First link is set")
            .that(&res)
            .is_ok()
            .map(|val| &val.first)
            .is_some();
        asserting("Prev link is set")
            .that(&res)
            .is_ok()
            .map(|val| &val.prev)
            .is_some();
        asserting("Next link is set")
            .that(&res)
            .is_ok()
            .map(|val| &val.next)
            .is_some();
        asserting("Last link is set")
            .that(&res)
            .is_ok()
            .map(|val| &val.last)
            .is_some();
    }
}

mod parser {
    /*
    Link: <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=15>; rel="next",
    <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=34>; rel="last",
    <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=1>; rel="first",
    <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=13>; rel="prev"
    */

    use nom::{
        bytes::complete::{tag, take_until},
        combinator::map_res,
        multi::separated_list,
        sequence::{delimited, separated_pair},
        IResult,
    };
    use std::convert::TryFrom;

    #[derive(Debug, PartialEq, Eq)]
    pub struct Link<'a> {
        pub url: &'a str,
        pub dir: Direction,
    }

    impl<'a> TryFrom<(&'a str, Direction)> for Link<'a> {
        type Error = &'static str;

        fn try_from(value: (&'a str, Direction)) -> Result<Self, Self::Error> {
            let (url, dir) = value;

            Ok(Link { url, dir })
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum Direction {
        First,
        Prev,
        Next,
        Last,
    }

    impl TryFrom<&str> for Direction {
        type Error = &'static str;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value.to_lowercase().as_str() {
                "first" => Ok(Self::First),
                "prev" => Ok(Self::Prev),
                "next" => Ok(Self::Next),
                "last" => Ok(Self::Last),
                _ => Err("unexpected link direction"),
            }
        }
    }

    pub fn links(input: &str) -> IResult<&str, Vec<Link>> {
        separated_list(tag(", "), link)(input)
    }

    fn link(input: &str) -> IResult<&str, Link> {
        map_res(separated_pair(url, tag("; "), dir), Link::try_from)(input)
    }

    fn url(input: &str) -> IResult<&str, &str> {
        delimited(tag("<"), take_until(">"), tag(">"))(input)
    }

    fn dir(input: &str) -> IResult<&str, Direction> {
        map_res(
            delimited(tag("rel=\""), take_until("\""), tag("\"")),
            Direction::try_from,
        )(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use spectral::prelude::*;

        #[test]
        fn links_ok() {
            let input = r#"<https://api.github.com/search/code?q=addClass+user%3Amozilla&page=15>; rel="next", <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=34>; rel="last", <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=1>; rel="first", <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=13>; rel="prev""#;
            let res = links(input);
            asserting("Parsing link-dirs")
                .that(&res)
                .is_ok()
                .map(|val| &val.1)
                .has_length(4);
        }

        #[test]
        fn link_ok() {
            let input = r#"<https://api.github.com/search/code?q=addClass+user%3Amozilla&page=15>; rel="next""#;
            let res = link(input);
            asserting("Parsing link-dir").that(&res).is_equal_to(Ok((
                "",
                Link {
                    url: "https://api.github.com/search/code?q=addClass+user%3Amozilla&page=15",
                    dir: Direction::Next,
                },
            )))
        }

        #[test]
        fn url_ok() {
            let input = r#"<https://api.github.com/search/code?q=addClass+user%3Amozilla&page=13>"#;
            let res = url(input);
            asserting("Parsing link").that(&res).is_equal_to(Ok((
                "",
                "https://api.github.com/search/code?q=addClass+user%3Amozilla&page=13",
            )))
        }

        #[test]
        fn dir_first() {
            let input = r#"rel="first""#;
            let res = dir(input);
            asserting("Parsing link direction 'first'")
                .that(&res)
                .is_equal_to(Ok(("", Direction::First)))
        }
        #[test]
        fn dir_prev() {
            let input = r#"rel="prev""#;
            let res = dir(input);
            asserting("Parsing link direction 'prev'")
                .that(&res)
                .is_equal_to(Ok(("", Direction::Prev)))
        }
        #[test]
        fn dir_next() {
            let input = r#"rel="next""#;
            let res = dir(input);
            asserting("Parsing link direction 'next'")
                .that(&res)
                .is_equal_to(Ok(("", Direction::Next)))
        }
        #[test]
        fn dir_last() {
            let input = r#"rel="last""#;
            let res = dir(input);
            asserting("Parsing link direction 'last'")
                .that(&res)
                .is_equal_to(Ok(("", Direction::Last)))
        }
    }
}

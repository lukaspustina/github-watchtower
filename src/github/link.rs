
/*
 * Link: <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=15>; rel="next",
  <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=34>; rel="last",
  <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=1>; rel="first",
  <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=13>; rel="prev"
*/

pub struct Links<'a> {
    first: Option<&'a str>,
    prev: Option<&'a str>,
    next: Option<&'a str>,
    last: Option<&'a str>,
}

mod parser {
    use nom::{
        IResult,
        bytes::complete::{tag, take_until},
        combinator::map_res,
        multi::separated_list,
        sequence::{delimited, separated_pair},
    };
    use std::convert::TryFrom;
  
    #[derive(Debug, PartialEq, Eq)]
    struct Link<'a> {
        url: &'a str,
        dir: Direction,
    }

    impl<'a> TryFrom<(&'a str, Direction)> for Link<'a> {
        type Error = &'static str;

        fn try_from(value: (&'a str, Direction)) -> Result<Self, Self::Error>  {
            let (url, dir) = value;

            Ok(
                Link {
                    url,
                    dir,
                }
            )
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum Direction {
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
                _ => Err("unexpected link direction")
            }
        }
    }

    fn link_dirs(input: &str) -> IResult<&str, Vec<Link>> {
        separated_list(tag(", "), link_dir)(input)
    }

    fn link_dir(input: &str) -> IResult<&str, Link> {
        map_res(
            separated_pair( link, tag("; "), dir),
            Link::try_from
        )(input)
    }

    fn link(input: &str) -> IResult<&str, &str> {
        delimited(tag("<"), take_until(">"), tag(">"))(input)
    }

    fn dir(input: &str) -> IResult<&str, Direction> {
        map_res(
            delimited(tag("rel=\""), take_until("\""), tag("\"")),
            Direction::try_from
        )(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use spectral::prelude::*;

        #[test]
        fn link_dirs_ok() {
            let input = r#"<https://api.github.com/search/code?q=addClass+user%3Amozilla&page=15>; rel="next", <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=34>; rel="last", <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=1>; rel="first", <https://api.github.com/search/code?q=addClass+user%3Amozilla&page=13>; rel="prev""#;
            let res = link_dirs(input);
            asserting("Parsing link-dirs")
                .that(&res)
                .is_ok()
                .map(|val| &val.1)
                .has_length(4);
        }

        #[test]
        fn link_dir_ok() {
            let input = r#"<https://api.github.com/search/code?q=addClass+user%3Amozilla&page=15>; rel="next""#;
            let res = link_dir(input);
            asserting("Parsing link-dir")
                .that(&res)
                .is_equal_to(Ok(("", Link { 
                    url: "https://api.github.com/search/code?q=addClass+user%3Amozilla&page=15",
                    dir: Direction::Next
                })))
        }

        #[test]
        fn link_ok() {
            let input = r#"<https://api.github.com/search/code?q=addClass+user%3Amozilla&page=13>"#;
            let res = link(input);
            asserting("Parsing link")
                .that(&res)
                .is_equal_to(Ok(("", "https://api.github.com/search/code?q=addClass+user%3Amozilla&page=13")))
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

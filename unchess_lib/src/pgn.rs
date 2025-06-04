//! Parsing for PGN notation

use nom::{
    IResult, Parser as _,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::{map_res, opt, value},
    error::Error,
};

use crate::default_types::SimpleSquare;

fn rank(input: &str) -> IResult<&str, u8> {
    map_res(one_of("12345678"), |c| {
        Ok::<u8, Error<&str>>(c.to_digit(10).unwrap() as u8 - 1)
    })
    .parse(input)
}

fn file(input: &str) -> IResult<&str, u8> {
    alt((
        value(0, char('a')),
        value(1, char('b')),
        value(2, char('c')),
        value(3, char('d')),
        value(4, char('e')),
        value(5, char('f')),
        value(6, char('g')),
        value(7, char('h')),
    ))
    .parse(input)
}

fn square(input: &str) -> IResult<&str, SimpleSquare> {
    let (input, (file, rank)) = (file, rank).parse(input)?;
    Ok((input, SimpleSquare::new(file, rank)))
}

#[derive(Debug)]
struct Disambiguation {
    file: Option<u8>,
    rank: Option<u8>,
}

fn disambiguation(input: &str) -> (&str, Disambiguation) {
    let (input, f) = opt(file).parse(input).unwrap();
    let (input, r) = opt(rank).parse(input).unwrap();
    (input, Disambiguation { file: f, rank: r })
}

#[derive(Debug)]
struct DisambiguatedMove {
    disambiguation: Disambiguation,
    takes: bool,
    dest: SimpleSquare,
}

fn move_with_disambiguation(input: &str) -> IResult<&str, DisambiguatedMove> {
    let (input, disambiguation) = disambiguation(input);
    let (input, takes) = opt(|s| tag("x").parse(s)).parse(input)?;
    let takes = takes.is_some();
    let (input, dest) = square.parse(input)?;
    Ok((
        input,
        DisambiguatedMove {
            disambiguation,
            takes,
            dest,
        },
    ))
}

fn move_no_disambiguation(input: &str) -> IResult<&str, DisambiguatedMove> {
    let (input, takes) = opt(|s| tag("x").parse(s)).parse(input)?;
    let takes = takes.is_some();
    let (input, dest) = square.parse(input)?;
    Ok((
        input,
        DisambiguatedMove {
            disambiguation: Disambiguation { file: None, rank: None },
            takes,
            dest,
        },
    ))
}

fn disambiguated_move(input: &str) -> IResult<&str, DisambiguatedMove> {
    alt((move_with_disambiguation, move_no_disambiguation)).parse(input)
}

#[cfg(test)]
mod tests {
    use crate::{notation, traits::ChessSquare as _};

    use super::*;
    use proptest::prelude::*;
    use std::fmt::Write as _;

    proptest! {
        #[test]
        fn good_squares(file in 0..=7u8, rank in 0..=7u8) {
            let s = SimpleSquare::new(file, rank);
            assert_eq!(square(&s.as_str()), Ok(("", s)));
        }

        #[test]
        fn bad_squares(s in "[i-z](0|9)") {
            square(&s).unwrap_err();
        }

        #[test]
        fn double_disamb(file in 0..=7u8, rank in 0..=7u8) {
            let s = SimpleSquare::new(file, rank);
            let out = disambiguation(&s.as_str()).1;
            assert_eq!((out.file, out.rank), (Some(file), Some(rank)));
        }

        #[test]
        fn file_disamb(file in 0..=7u8) {
            let out = disambiguation(&notation::file(file).unwrap().to_string()).1;
            assert_eq!((out.file, out.rank), (Some(file), None));
        }

        #[test]
        fn rank_disamb(rank in 0..=7u8) {
            let out = disambiguation(&notation::rank(rank).unwrap().to_string()).1;
            assert_eq!((out.file, out.rank), (None, Some(rank)));
        }

        #[test]
        fn move_with_disamb(disamb_file in 0..=7u8, disamb_rank in 0..=7u8, takes in any::<bool>(), file in 0..=7u8, rank in 0..=7u8) {
            let dest = SimpleSquare::new(file, rank);
            let src = SimpleSquare::new(disamb_file, disamb_rank);
            let mut s = src.as_str();
            if takes {
                s.push('x');
            }
            write!(s, "{dest}").unwrap();
            let out = disambiguated_move(&s).unwrap().1;
            assert_eq!(out.dest, dest);
            assert_eq!(out.takes, takes);
            assert_eq!((out.disambiguation.file, out.disambiguation.rank), (Some(disamb_file), Some(disamb_rank)));
        }

        #[test]
        fn move_no_disamb(takes in any::<bool>(), file in 0..=7u8, rank in 0..=7u8) {
            let dest = SimpleSquare::new(file, rank);
            let mut s = String::new();
            if takes {
                s.push('x');
            }
            write!(s, "{dest}").unwrap();
            let out = disambiguated_move(&s).unwrap().1;
            assert_eq!(out.dest, dest);
            assert_eq!(out.takes, takes);
            assert_eq!((out.disambiguation.file, out.disambiguation.rank), (None, None));
        }
    }
}

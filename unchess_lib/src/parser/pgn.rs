//! Parsing for PGN notation
#![allow(clippy::type_complexity)]

use nom::{
    IResult, Parser as _,
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{char, digit1, multispace0, multispace1, one_of},
    combinator::{map_res, opt, value},
    multi::{many0, many1},
    sequence::{delimited, pair, separated_pair},
};

use crate::{
    enums::{AmbiguousMove, CastlingSide, MoveAction, PieceKind},
    notation::{char_to_file, char_to_rank},
    simple_types::SimpleSquare,
};

fn rank(input: &str) -> IResult<&str, u8> {
    map_res(one_of("12345678"), char_to_rank).parse(input)
}

fn file(input: &str) -> IResult<&str, u8> {
    map_res(one_of("abcdefgh"), char_to_file).parse(input)
}

fn square(input: &str) -> IResult<&str, SimpleSquare> {
    let (input, (file, rank)) = (file, rank).parse(input)?;
    Ok((input, SimpleSquare::new(file, rank)))
}

fn dest(input: &str) -> IResult<&str, (bool, SimpleSquare)> {
    let (input, takes) = opt(tag("x")).parse(input)?;
    let (input, square) = square(input)?;
    Ok((input, (takes.is_some(), square)))
}

fn disambiguated_move(input: &str) -> IResult<&str, (Option<u8>, Option<u8>, bool, SimpleSquare)> {
    alt((
        |input| {
            let (input, file) = opt(file).parse(input)?;
            let (input, rank) = opt(rank).parse(input)?;
            let (input, dest) = dest.parse(input)?;
            Ok((input, (file, rank, dest.0, dest.1)))
        },
        |input| {
            let (input, dest) = dest.parse(input)?;
            Ok((input, (None, None, dest.0, dest.1)))
        },
    ))
    .parse(input)
}

fn piece(input: &str) -> IResult<&str, PieceKind> {
    let (input, piece_kind) = opt(map_res(one_of("QKNBR"), PieceKind::try_from)).parse(input)?;
    Ok((input, piece_kind.unwrap_or(PieceKind::Pawn)))
}

fn promotion(input: &str) -> IResult<&str, PieceKind> {
    let (input, _) = tag("=")(input)?;
    piece(input)
}

fn action(input: &str) -> IResult<&str, MoveAction> {
    alt((
        value(MoveAction::Check, char('+')),
        value(MoveAction::Checkmate, char('#')),
    ))
    .parse(input)
}

fn normal_move(input: &str) -> IResult<&str, AmbiguousMove> {
    let (input, piece_kind) = piece(input)?;
    let (input, (src_file, src_rank, takes, dest)) = disambiguated_move(input)?;
    let (input, promote_to) = opt(promotion).parse(input)?;
    let (input, action) = opt(action).parse(input)?;
    Ok((
        input,
        AmbiguousMove::Normal {
            piece_kind,
            src_file,
            src_rank,
            takes,
            dest,
            promote_to,
            action,
        },
    ))
}

/// Parse PGN standard chess move
pub fn chess_move(input: &str) -> IResult<&str, AmbiguousMove> {
    alt((
        normal_move,
        value(
            AmbiguousMove::Castle {
                side: CastlingSide::QueenSide,
            },
            tag("O-O-O"),
        ),
        value(
            AmbiguousMove::Castle {
                side: CastlingSide::KingSide,
            },
            tag("O-O"),
        ),
    ))
    .parse(input)
}

fn eol_comment(input: &str) -> IResult<&str, ()> {
    value(
        (), // Output is thrown away.
        pair(char(';'), is_not("\n\r")),
    )
    .parse(input)
}

fn enclosed_comment(input: &str) -> IResult<&str, ()> {
    value(
        (), // Output is thrown away.
        (tag("{"), take_until("}"), tag("}")),
    )
    .parse(input)
}

fn tag_pair(input: &str) -> IResult<&str, (&str, &str)> {
    let (input, pair) = delimited(char('['), is_not("]"), char(']')).parse(input)?;
    let (_, (key, value)) = separated_pair(is_not(" "), multispace0, is_not("]")).parse(pair)?;
    Ok((input, (key, value)))
}

fn move_number(input: &str) -> IResult<&str, ()> {
    let (input, _) = digit1(input)?;
    let (input, _) = many1(tag(".")).parse(input)?;
    Ok((input, ()))
}

fn move_without_comments(input: &str) -> IResult<&str, AmbiguousMove> {
    let (input, _) = many0(alt((
        enclosed_comment,
        eol_comment,
        |s| Ok((multispace1(s)?.0, ())),
        move_number,
    )))
    .parse(input)?;
    chess_move(input)
}

#[allow(clippy::type_complexity)]
pub fn pgn(input: &str) -> IResult<&str, (Vec<(&str, &str)>, Vec<AmbiguousMove>)> {
    let (input, tag_pairs) = many0(|s| {
        let (s, _) = multispace0(s)?;
        tag_pair(s)
    })
    .parse(input)?;
    let (input, moves) = many1(move_without_comments).parse(input)?;
    Ok((input, ((tag_pairs), moves)))
}

#[cfg(test)]
mod tests {
    use crate::{enums::AmbiguousMove, traits::ChessSquare as _};

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
        fn move_with_disamb(disamb_file in 0..=7u8, disamb_rank in 0..=7u8, takes in any::<bool>(), file in 0..=7u8, rank in 0..=7u8) {
            let dest = SimpleSquare::new(file, rank);
            let src = SimpleSquare::new(disamb_file, disamb_rank);
            let mut s = src.as_str();
            if takes {
                s.push('x');
            }
            write!(s, "{dest}").unwrap();
            let out = disambiguated_move(&s).unwrap().1;
            assert_eq!(out.3, dest);
            assert_eq!(out.2, takes);
            assert_eq!((out.0, out.1), (Some(disamb_file), Some(disamb_rank)));
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
            assert_eq!(out.3, dest);
            assert_eq!(out.2, takes);
            assert_eq!((out.0, out.1), (None, None));
        }

        #[test]
        fn all_ambiguous_moves(amb_move in AmbiguousMove::strategy()) {
            assert_eq!(chess_move(&amb_move.as_pgn_str()).unwrap(), ("", amb_move));
        }
    }
}

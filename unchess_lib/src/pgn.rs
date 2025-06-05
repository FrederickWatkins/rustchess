//! Parsing for PGN notation

use nom::{
    IResult, Parser as _,
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{char, digit1, multispace0, multispace1, one_of},
    combinator::{map_res, opt, value},
    error::Error,
    multi::{many0, many1},
    sequence::{delimited, pair, separated_pair},
};

use crate::{
    default_types::SimpleSquare,
    enums::{AmbiguousMove, CastlingSide, MoveAction, PieceKind},
};

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

fn disambiguation(input: &str) -> IResult<&str, Disambiguation> {
    let (input, f) = opt(file).parse(input)?;
    let (input, r) = opt(rank).parse(input)?;
    Ok((input, Disambiguation { file: f, rank: r }))
}

#[derive(Debug)]
struct DisambiguatedMove {
    disambiguation: Disambiguation,
    takes: bool,
    dest: SimpleSquare,
}

fn move_with_disambiguation(input: &str) -> IResult<&str, DisambiguatedMove> {
    let (input, disambiguation) = disambiguation(input)?;
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

fn piece(input: &str) -> IResult<&str, PieceKind> {
    let (input, piece_kind) = opt(alt((
        value(PieceKind::Bishop, char('B')),
        value(PieceKind::King, char('K')),
        value(PieceKind::Knight, char('N')),
        value(PieceKind::Queen, char('Q')),
        value(PieceKind::Rook, char('R')),
    )))
    .parse(input)?;
    Ok((input, piece_kind.unwrap_or(PieceKind::Pawn)))
}

fn promotion(input: &str) -> IResult<&str, PieceKind> {
    let (input, _) = tag("=")(input)?;
    alt((
        value(PieceKind::Knight, char('N')),
        value(PieceKind::Queen, char('Q')),
        value(PieceKind::Rook, char('R')),
        value(PieceKind::Bishop, char('B')),
    ))
    .parse(input)
}

fn move_action(input: &str) -> IResult<&str, MoveAction> {
    alt((
        value(MoveAction::Check, char('+')),
        value(MoveAction::Checkmate, char('#')),
    ))
    .parse(input)
}

/// Parse PGN standard chess move
pub fn chess_move(input: &str) -> IResult<&str, AmbiguousMove> {
    alt((
        |input| {
            let (input, (piece_kind, disamb_move, promote_to, action)) =
                (piece, disambiguated_move, opt(promotion), opt(move_action)).parse(input)?;
            Ok((
                input,
                AmbiguousMove::Normal {
                    piece_kind,
                    src_file: disamb_move.disambiguation.file,
                    src_rank: disamb_move.disambiguation.rank,
                    takes: disamb_move.takes,
                    dest: disamb_move.dest,
                    promote_to,
                    action,
                },
            ))
        },
        |input| {
            Ok((
                tag("O-O-O")(input)?.0,
                AmbiguousMove::Castle {
                    side: CastlingSide::QueenSide,
                },
            ))
        },
        |input| {
            Ok((
                tag("O-O")(input)?.0,
                AmbiguousMove::Castle {
                    side: CastlingSide::KingSide,
                },
            ))
        },
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

fn move_decorator(input: &str) -> IResult<&str, ()> {
    alt((|s| Ok((digit1(s)?.0, ())), |s| Ok((char('.')(s)?.0, ())))).parse(input)
}

fn move_without_comments(input: &str) -> IResult<&str, AmbiguousMove> {
    let (input, _) = many0(alt((
        enclosed_comment,
        eol_comment,
        |s| Ok((multispace1(s)?.0, ())),
        move_decorator,
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
    use crate::{
        enums::{AmbiguousMove, CastlingSide},
        notation,
        traits::ChessSquare as _,
    };

    use super::*;
    use proptest::{option::of, prelude::*};
    use std::fmt::Write as _;

    fn amb_move_strategy() -> impl Strategy<Value = AmbiguousMove> {
        let piece_kind = prop_oneof![
            Just(PieceKind::Pawn),
            Just(PieceKind::Rook),
            Just(PieceKind::Knight),
            Just(PieceKind::King),
            Just(PieceKind::Queen,),
            Just(PieceKind::Bishop),
        ];
        let src_file = of(any::<u8>().prop_filter("Valid range for file", |x| (0..=7).contains(x)));
        let src_rank = of(any::<u8>().prop_filter("Valid range for file", |x| (0..=7).contains(x)));
        let castle = any::<bool>();
        let castling_side = prop_oneof![Just(CastlingSide::KingSide), Just(CastlingSide::QueenSide),];
        let takes = any::<bool>();
        let dest_file = any::<u8>().prop_filter("Valid range for file", |x| (0..=7).contains(x));
        let dest_rank = any::<u8>().prop_filter("Valid range for file", |x| (0..=7).contains(x));
        let promote_to = of(prop_oneof![
            Just(PieceKind::Rook),
            Just(PieceKind::Knight),
            Just(PieceKind::Queen,),
            Just(PieceKind::Bishop),
        ]);
        let action = of(prop_oneof![Just(MoveAction::Check), Just(MoveAction::Checkmate)]);
        (
            castle,
            castling_side,
            piece_kind,
            src_file,
            src_rank,
            takes,
            dest_file,
            dest_rank,
            promote_to,
            action,
        )
            .prop_map(
                |(
                    castle,
                    castling_side,
                    piece_kind,
                    src_file,
                    src_rank,
                    takes,
                    dest_file,
                    dest_rank,
                    promote_to,
                    action,
                )| {
                    if castle {
                        AmbiguousMove::Castle { side: castling_side }
                    } else {
                        AmbiguousMove::Normal {
                            piece_kind,
                            src_file,
                            src_rank,
                            takes,
                            dest: SimpleSquare::new(dest_file, dest_rank),
                            promote_to,
                            action,
                        }
                    }
                },
            )
    }

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
            let out = disambiguation(&s.as_str())?.1;
            assert_eq!((out.file, out.rank), (Some(file), Some(rank)));
        }

        #[test]
        fn file_disamb(file in 0..=7u8) {
            let out = disambiguation(&notation::file(file).unwrap().to_string())?.1;
            assert_eq!((out.file, out.rank), (Some(file), None));
        }

        #[test]
        fn rank_disamb(rank in 0..=7u8) {
            let out = disambiguation(&notation::rank(rank).unwrap().to_string())?.1;
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

        #[test]
        fn all_ambiguous_moves(amb_move in amb_move_strategy()) {
            assert_eq!(chess_move(&amb_move.as_pgn_str()).unwrap(), ("", amb_move));
        }
    }
}

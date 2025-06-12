use itertools::Itertools as _;
use nom::{
    Err, IResult, Parser as _,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0, one_of, u32, usize},
    combinator::{map_res, opt, value},
    error,
    multi::{many1, separated_list1},
};
#[cfg(test)]
use proptest::prelude::Strategy;

use std::fmt::Write as _;

use crate::{
    enums::{CastlingSide, PieceColour, PieceKind},
    parser::pgn::square,
    simple_types::{SimplePiece, SimpleSquare},
};

fn white_piece(input: &str) -> IResult<&str, SimplePiece> {
    let (input, piece_kind) = map_res(one_of("QKNBRP"), PieceKind::try_from).parse(input)?;
    Ok((input, SimplePiece::new(piece_kind, PieceColour::White)))
}

fn black_piece(input: &str) -> IResult<&str, SimplePiece> {
    let (input, piece_kind) =
        map_res(one_of("qknbrp"), |c| PieceKind::try_from(c.to_ascii_uppercase())).parse(input)?;
    Ok((input, SimplePiece::new(piece_kind, PieceColour::Black)))
}

fn piece(input: &str) -> IResult<&str, SimplePiece> {
    alt((white_piece, black_piece)).parse(input)
}

fn turn(input: &str) -> IResult<&str, PieceColour> {
    alt((value(PieceColour::White, tag("w")), value(PieceColour::Black, tag("b")))).parse(input)
}

fn rank(mut input: &str) -> IResult<&str, [Option<SimplePiece>; 8]> {
    let mut i = 0;
    let mut out = [None; 8];
    while i < out.len() {
        input = if let Ok((input, piece)) = piece(input) {
            out[i] = Some(piece);
            i += 1;
            input
        } else {
            let (input, empty_squares) = usize(input)?;
            i += empty_squares;
            input
        };
    }
    Ok((input, out))
}

fn board_layout(input: &str) -> IResult<&str, Box<[[Option<SimplePiece>; 8]; 8]>> {
    let (input, pieces) = separated_list1(tag("/"), rank).parse(input)?;
    if let Some(pieces) = pieces.into_iter().collect_array() {
        Ok((input, Box::new(pieces)))
    } else {
        Err(Err::Error(error::Error {
            input,
            code: error::ErrorKind::TooLarge,
        }))
    }
}

fn castling_rights(input: &str) -> IResult<&str, [bool; 4]> {
    let (input, castles) = many1(alt((
        |i| {
            let (i, _) = char('K')(i)?;
            Ok((i, Some((CastlingSide::KingSide, PieceColour::White))))
        },
        |i| {
            let (i, _) = char('Q')(i)?;
            Ok((i, Some((CastlingSide::QueenSide, PieceColour::White))))
        },
        |i| {
            let (i, _) = char('k')(i)?;
            Ok((i, Some((CastlingSide::KingSide, PieceColour::Black))))
        },
        |i| {
            let (i, _) = char('q')(i)?;
            Ok((i, Some((CastlingSide::QueenSide, PieceColour::Black))))
        },
        |i| {
            let (i, _) = char('-')(i)?;
            Ok((i, None))
        },
    )))
    .parse(input)?;
    Ok((
        input,
        [
            castles.contains(&Some((CastlingSide::KingSide, PieceColour::White))),
            castles.contains(&Some((CastlingSide::QueenSide, PieceColour::White))),
            castles.contains(&Some((CastlingSide::KingSide, PieceColour::Black))),
            castles.contains(&Some((CastlingSide::QueenSide, PieceColour::Black))),
        ],
    ))
}

fn en_passant(input: &str) -> IResult<&str, Option<SimpleSquare>> {
    alt((
        |i| {
            let (i, square) = square(i)?;
            Ok((i, Some(square)))
        },
        |i| {
            let (i, _) = char('-')(i)?;
            Ok((i, None))
        },
    ))
    .parse(input)
}

pub fn fen(input: &str) -> IResult<&str, Fen> {
    let (input, _) = multispace0(input)?;
    let (input, layout) = board_layout(input)?;
    let (input, _) = multispace0(input)?;
    let (input, turn) = turn(input)?;
    let (input, _) = multispace0(input)?;
    let (input, castling_rights) = castling_rights(input)?;
    let (input, _) = multispace0(input)?;
    let (input, en_passant) = en_passant(input)?;
    let (input, _) = multispace0(input)?;
    let (input, halfmove_clock) = opt(u32).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, fullmove_number) = opt(u32).parse(input)?;
    Ok((
        input,
        Fen {
            layout,
            turn,
            castling_rights,
            en_passant,
            halfmove_clock: halfmove_clock.unwrap_or(0),
            fullmove_number: fullmove_number.unwrap_or(0),
        },
    ))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fen {
    pub layout: Box<[[Option<SimplePiece>; 8]; 8]>,
    pub turn: PieceColour,
    pub castling_rights: [bool; 4],
    pub en_passant: Option<SimpleSquare>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Fen {
    pub fn to_str(&self) -> String {
        const CASTLING_LETTERS: [char; 4] = ['K', 'Q', 'k', 'q'];
        let mut s = String::new();
        s.push_str(&layout_to_str(&self.layout));
        s.push(' ');
        match self.turn {
            PieceColour::Black => s.push('b'),
            PieceColour::White => s.push('w'),
        }
        s.push(' ');
        for (i, castling_right) in self.castling_rights.iter().enumerate() {
            if *castling_right {
                s.push(CASTLING_LETTERS[i]);
            }
        }
        if self.castling_rights.iter().all(|b| !b) {
            s.push('-');
        }
        s.push(' ');
        if let Some(square) = self.en_passant {
            s.push_str(&square.to_string());
        } else {
            s.push('-');
        }
        s.push(' ');
        write!(s, "{} {}", self.halfmove_clock, self.fullmove_number).unwrap();
        s
    }

    #[cfg(test)]
    pub fn strategy() -> impl Strategy<Value = Self> {
        use proptest::{array::uniform4, collection::vec, option::of, prelude::any};

        let layout = vec(vec(of(SimplePiece::strategy()), 8), 8);
        let turn = PieceColour::strategy();
        let castling_rights = uniform4(any::<bool>());
        let en_passant = of(SimpleSquare::strategy());
        let halfmove_clock = any::<u32>();
        let fullmove_number = any::<u32>();
        (
            layout,
            turn,
            castling_rights,
            en_passant,
            halfmove_clock,
            fullmove_number,
        )
            .prop_map(
                |(layout, turn, castling_rights, en_passant, halfmove_clock, fullmove_number)| {
                    let layout: Box<[[Option<SimplePiece>; 8]; 8]> = Box::new(
                        layout
                            .iter()
                            .map(|arr| arr.iter().copied().collect_array().unwrap())
                            .collect_array()
                            .unwrap(),
                    );
                    Self {
                        layout,
                        turn,
                        castling_rights,
                        en_passant,
                        halfmove_clock,
                        fullmove_number,
                    }
                },
            )
    }
}

fn rank_to_str(rank: &[Option<SimplePiece>; 8]) -> String {
    let mut s = String::new();
    let mut empty_squares = 0usize;
    for piece in rank {
        if let Some(p) = piece {
            if empty_squares > 0 {
                write!(s, "{empty_squares}").unwrap();
                empty_squares = 0;
            }
            s.push(char::from(*p));
        } else {
            empty_squares += 1;
        }
    }
    if empty_squares > 0 {
        write!(s, "{empty_squares}").unwrap();
    }
    s
}

fn layout_to_str(layout: &[[Option<SimplePiece>; 8]; 8]) -> String {
    let mut s = String::new();
    for rank in layout {
        s.push_str(&rank_to_str(rank));
        s.push('/');
    }
    s.pop();
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simple_types::SimplePiece;
    use crate::traits::ChessPiece as _;
    use proptest::array::uniform8;
    use proptest::collection::vec;
    use proptest::option::of;
    use proptest::proptest;

    proptest! {
        #[test]
        fn pieces(p in SimplePiece::strategy()) {
            assert_eq!(piece(&p.as_fen().to_string()).unwrap().1, p);
        }

        #[test]
        fn ranks(r in uniform8(of(SimplePiece::strategy()))) {
            assert_eq!(rank(&rank_to_str(&r)).unwrap().1, r);
        }

        #[test]
        fn turns(t in PieceColour::strategy()) {
            let s = match t {
                PieceColour::Black => "b",
                PieceColour::White => "w",
            };
            assert_eq!(turn(s).unwrap().1, t);
        }

        #[test]
        fn layouts(l in vec(vec(of(SimplePiece::strategy()), 8), 8)) {
            #[rustfmt::skip]
            let layout: Box<[[Option<SimplePiece>; 8]; 8]> = Box::new(l.iter().map(|arr| arr.iter().copied().collect_array().unwrap()).collect_array().unwrap());
            let s = layout_to_str(&layout);
            assert_eq!(board_layout(&s).unwrap(), ("", layout));
        }

        #[test]
        fn fens(f in Fen::strategy()) {
            assert_eq!(fen(&f.to_str()).unwrap(), ("", f));
        }
    }
}

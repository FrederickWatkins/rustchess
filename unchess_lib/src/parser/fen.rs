use nom::{
    IResult, Parser as _,
    branch::alt,
    bytes::complete::tag,
    character::complete::{one_of, usize},
    combinator::{map_res, value},
};

use crate::{
    enums::{PieceColour, PieceKind},
    simple_types::SimplePiece,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simple_types::SimplePiece;
    use crate::traits::ChessPiece as _;
    use proptest::array::uniform8;
    use proptest::option::of;
    use proptest::proptest;

    fn rank_to_str(rank: [Option<SimplePiece>; 8]) -> String {
        let mut s = String::new();
        let mut empty_squares = 0usize;
        for piece in rank {
            if let Some(p) = piece {
                if empty_squares > 0 {
                    s.push_str(&format!("{empty_squares}"));
                    empty_squares = 0;
                }
                s.push(char::from(p))
            } else {
                empty_squares += 1;
            }
        }
        if empty_squares > 0 {
            s.push_str(&format!("{empty_squares}"));
        }
        s
    }

    proptest! {
        #[test]
        fn pieces(p in SimplePiece::strategy()) {
            assert_eq!(piece(&p.as_fen().to_string()).unwrap().1, p);
        }

        #[test]
        fn ranks(r in uniform8(of(SimplePiece::strategy()))) {
            assert_eq!(rank(&rank_to_str(r)).unwrap().1, r);
        }

        #[test]
        fn turns(t in PieceColour::strategy()) {
            let s = match t {
                PieceColour::Black => "b",
                PieceColour::White => "w",
            };
            assert_eq!(turn(s).unwrap().1, t);
        }
    }
}

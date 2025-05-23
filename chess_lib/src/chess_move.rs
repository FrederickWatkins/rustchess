use crate::piece::*;

pub struct AmbiguousMove {
    pub end: Position,
    pub start: (Option<i64>, Option<i64>),
    pub kind: PieceKind,
    // TODO: pub takes: bool,
}

pub struct UnambiguousMove {
    pub end: Position,
    pub start: Position,
    // TODO: pub takes: Option<PieceKind>,
}

impl UnambiguousMove {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end}
    }
}
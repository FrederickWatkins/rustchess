use std::{
    fmt::Display,
    ops::{Add, AddAssign},
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Position(pub i8, pub i8);

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Position(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        *self = rhs + *self;
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct ChessMove(pub Position, pub Position);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pos_display() {
        assert_eq!(format!("{}", Position(5, 3)), *"(5, 3)");
    }
}

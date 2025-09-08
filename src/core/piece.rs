#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    #[inline]
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

pub struct Piece {
    pub color: Color,
    pub kind: PieceType,
}

impl PieceType {
    /// Parse from a promotion character in UCI notation ('q','r','b','n')
    pub fn from_uci(ch: char) -> Option<Self> {
        match ch {
            'q' => Some(PieceType::Queen),
            'r' => Some(PieceType::Rook),
            'b' => Some(PieceType::Bishop),
            'n' => Some(PieceType::Knight),
            _ => None,
        }
    }
}

#[inline]
pub const fn piece_index(color: Color, kind: PieceType) -> usize {
    match (color, kind) {
        (Color::White, PieceType::Pawn) => 0,
        (Color::White, PieceType::Knight) => 1,
        (Color::White, PieceType::Bishop) => 2,
        (Color::White, PieceType::Rook) => 3,
        (Color::White, PieceType::Queen) => 4,
        (Color::White, PieceType::King) => 5,
        (Color::Black, PieceType::Pawn) => 6,
        (Color::Black, PieceType::Knight) => 7,
        (Color::Black, PieceType::Bishop) => 8,
        (Color::Black, PieceType::Rook) => 9,
        (Color::Black, PieceType::Queen) => 10,
        (Color::Black, PieceType::King) => 11,
    }
}

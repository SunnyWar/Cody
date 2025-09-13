// src/core/piece.rs

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PieceKind {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    White = 0,
    Black = 1,
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
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    WhitePawn = 0,
    WhiteKnight = 1,
    WhiteBishop = 2,
    WhiteRook = 3,
    WhiteQueen = 4,
    WhiteKing = 5,
    BlackPawn = 6,
    BlackKnight = 7,
    BlackBishop = 8,
    BlackRook = 9,
    BlackQueen = 10,
    BlackKing = 11,
    None = 12,
}

impl Piece {
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn from_parts(color: Color, kind: Option<PieceKind>) -> Self {
        match kind {
            Some(k) => {
                let base = color as u8 * 6;
                Self::from_index(base + k as u8)
            }
            None => Self::None,
        }
    }

    const fn from_index(idx: u8) -> Self {
        match idx {
            0 => Self::WhitePawn,
            1 => Self::WhiteKnight,
            2 => Self::WhiteBishop,
            3 => Self::WhiteRook,
            4 => Self::WhiteQueen,
            5 => Self::WhiteKing,
            6 => Self::BlackPawn,
            7 => Self::BlackKnight,
            8 => Self::BlackBishop,
            9 => Self::BlackRook,
            10 => Self::BlackQueen,
            11 => Self::BlackKing,
            12 => Self::None,
            _ => panic!("Invalid piece index"),
        }
    }

    #[inline]
    pub const fn color(self) -> Color {
        if (self as u8) < 6 {
            Color::White
        } else {
            Color::Black
        }
    }

    #[inline]
    pub const fn kind(self) -> PieceKind {
        unsafe { std::mem::transmute((self as u8) % 6) }
    }
}

impl Piece {
    #[inline]
    pub const fn to_char(self) -> char {
        use Piece::*;
        match self {
            WhitePawn => 'P',
            WhiteKnight => 'N',
            WhiteBishop => 'B',
            WhiteRook => 'R',
            WhiteQueen => 'Q',
            WhiteKing => 'K',
            BlackPawn => 'p',
            BlackKnight => 'n',
            BlackBishop => 'b',
            BlackRook => 'r',
            BlackQueen => 'q',
            BlackKing => 'k',
            None => ' ', // TODO - is this an error?
        }
    }
}

impl Piece {
    pub fn from_char(c: char) -> Option<Self> {
        use Piece::*;
        match c {
            'P' => Some(WhitePawn),
            'N' => Some(WhiteKnight),
            'B' => Some(WhiteBishop),
            'R' => Some(WhiteRook),
            'Q' => Some(WhiteQueen),
            'K' => Some(WhiteKing),
            'p' => Some(BlackPawn),
            'n' => Some(BlackKnight),
            'b' => Some(BlackBishop),
            'r' => Some(BlackRook),
            'q' => Some(BlackQueen),
            'k' => Some(BlackKing),
            _ => Some(None),
        }
    }
}

impl PieceKind {
    /// Parse from a promotion character in UCI notation ('q','r','b','n')
    pub fn from_uci(ch: char) -> Option<Self> {
        match ch {
            'q' => Some(PieceKind::Queen),
            'r' => Some(PieceKind::Rook),
            'b' => Some(PieceKind::Bishop),
            'n' => Some(PieceKind::Knight),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_opposite() {
        assert_eq!(Color::White.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::White);
    }

    #[test]
    fn test_piece_from_parts() {
        assert_eq!(
            Piece::from_parts(Color::White, Some(PieceKind::Queen)),
            Piece::WhiteQueen
        );
        assert_eq!(
            Piece::from_parts(Color::Black, Some(PieceKind::Knight)),
            Piece::BlackKnight
        );
        assert_eq!(Piece::from_parts(Color::White, None), Piece::None);
    }

    #[test]
    fn test_piece_color_and_kind() {
        assert_eq!(Piece::WhiteBishop.color(), Color::White);
        assert_eq!(Piece::BlackRook.color(), Color::Black);
        assert_eq!(Piece::WhiteKnight.kind(), PieceKind::Knight);
        assert_eq!(Piece::BlackQueen.kind(), PieceKind::Queen);
    }

    #[test]
    fn test_piece_to_char() {
        assert_eq!(Piece::WhitePawn.to_char(), 'P');
        assert_eq!(Piece::BlackKnight.to_char(), 'n');
        assert_eq!(Piece::None.to_char(), ' ');
    }

    #[test]
    fn test_piece_from_char() {
        assert_eq!(Piece::from_char('Q'), Some(Piece::WhiteQueen));
        assert_eq!(Piece::from_char('k'), Some(Piece::BlackKing));
        assert_eq!(Piece::from_char('x'), Some(Piece::None)); // Possibly unexpected
    }

    #[test]
    fn test_piece_index_consistency() {
        for i in 0..=11 {
            let piece = unsafe { std::mem::transmute::<u8, Piece>(i) };
            assert_eq!(piece.index(), i as usize);
        }
    }

    #[test]
    fn test_piecekind_from_uci() {
        assert_eq!(PieceKind::from_uci('q'), Some(PieceKind::Queen));
        assert_eq!(PieceKind::from_uci('n'), Some(PieceKind::Knight));
        assert_eq!(PieceKind::from_uci('x'), None);
    }

    #[test]
    #[should_panic(expected = "Invalid piece index")]
    fn test_invalid_piece_index_panics() {
        let _ = Piece::from_index(13);
    }
}

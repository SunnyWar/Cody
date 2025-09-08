#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Square {
    A1,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
}

impl Square {
    /// Create a Square from file ('a'..'h') and rank ('1'..'8') chars.
    pub fn from_coords(file: char, rank: char) -> Option<Self> {
        if !('a'..='h').contains(&file) || !('1'..='8').contains(&rank) {
            return None;
        }
        let file_idx = (file as u8) - b'a';
        let rank_idx = (rank as u8) - b'1';
        let idx = rank_idx * 8 + file_idx;
        // Safe because idx is guaranteed 0..63
        Some(unsafe { std::mem::transmute::<u8, Square>(idx) })
    }

    pub fn file(self) -> u8 {
        (self as u8) % 8
    }

    pub fn rank(self) -> u8 {
        (self as u8) / 8
    }

    pub fn to_uci(self) -> String {
        let file_char = (b'a' + self.file()) as char;
        let rank_char = (b'1' + self.rank()) as char;
        format!("{}{}", file_char, rank_char)
    }
}

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub from: u8,      // 0..63
    pub to: u8,        // 0..63
    pub promotion: u8, // 0..5 for piece type, 255 = none
    pub flags: u8,     // bit flags for special moves
}

impl Move {
    pub fn new(from: u8, to: u8) -> Self {
        Move {
            from,
            to,
            promotion: 255, // 255 = none
            flags: 0,       // no special flags
        }
    }

    pub fn null() -> Self {
        Move {
            from: 0,
            to: 0,
            promotion: 255, // none
            flags: 0,
        }
    }
}

impl Move {
    pub fn from_square(&self) -> String {
        square_to_string(self.from)
    }

    pub fn to_square(&self) -> String {
        square_to_string(self.to)
    }
}

fn square_to_string(sq: u8) -> String {
    let file = sq % 8;
    let rank = sq / 8;
    let file_char = (b'a' + file) as char;
    let rank_char = (b'1' + rank) as char;
    format!("{}{}", file_char, rank_char)
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from_square(), self.to_square())
    }
}

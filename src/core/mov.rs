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
}

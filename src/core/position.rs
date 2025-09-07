use crate::core::bitboard::bit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub from: u8,      // 0..63
    pub to: u8,        // 0..63
    pub promotion: u8, // 0..5 for piece type, 255 = none
    pub flags: u8,     // bit flags for special moves
}

// Flags
pub const FLAG_CAPTURE: u8 = 1 << 0;
pub const FLAG_KINGSIDE_CASTLE: u8 = 1 << 1;
pub const FLAG_QUEENSIDE_CASTLE: u8 = 1 << 2;
pub const FLAG_EN_PASSANT: u8 = 1 << 3;
pub const FLAG_PROMOTION: u8 = 1 << 4;

#[derive(Clone)]
pub struct Position {
    /// One bitboard per piece type & color:
    /// 0..5 = white P,N,B,R,Q,K
    /// 6..11 = black P,N,B,R,Q,K
    pub pieces: [u64; 12],

    /// Occupancy: [white, black, both]
    pub occupancy: [u64; 3],

    /// 0 = white, 1 = black
    pub side_to_move: u8,

    /// Castling rights: bit 0 = white kingside, bit 1 = white queenside,
    /// bit 2 = black kingside, bit 3 = black queenside
    pub castling_rights: u8,

    /// En passant target square (0..63) or 64 if none
    pub ep_square: u8,

    /// Halfmove clock for 50‑move rule
    pub halfmove_clock: u8,

    /// Fullmove number (starts at 1, increments after black’s move)
    pub fullmove_number: u16,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            pieces: [0; 12],
            occupancy: [0; 3],
            side_to_move: 0,
            castling_rights: 0,
            ep_square: 64,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }
}

impl Position {
    #[inline]
    fn set_piece(&mut self, sq: u8, piece_index: usize) {
        let bit = 1u64 << sq;
        self.pieces[piece_index] |= bit;
        let color = piece_index / 6;
        self.occupancy[color] |= bit;
        self.occupancy[2] |= bit;
    }

    #[inline]
    fn clear_square(&mut self, sq: u8) {
        let mask = !(1u64 << sq);
        for bb in &mut self.pieces {
            *bb &= mask;
        }
        self.occupancy[0] &= mask;
        self.occupancy[1] &= mask;
        self.occupancy[2] &= mask;
    }
}

impl Position {
    pub fn from_fen(fen: &str) -> Self {
        let mut pos = Position::default();
        let parts: Vec<&str> = fen.split_whitespace().collect();
        let board_part = parts[0];
        let side_part = parts[1];
        let castling_part = parts[2];
        let ep_part = parts[3];
        let halfmove_part = parts[4];
        let fullmove_part = parts[5];

        let mut sq: i32 = 56; // a8
        for ch in board_part.chars() {
            match ch {
                '/' => sq -= 16, // drop one rank
                '1'..='8' => sq += ch.to_digit(10).unwrap() as i32,
                _ => {
                    let (piece_index, _) = match ch {
                        'P' => (0, 0),
                        'N' => (1, 0),
                        'B' => (2, 0),
                        'R' => (3, 0),
                        'Q' => (4, 0),
                        'K' => (5, 0),
                        'p' => (6, 1),
                        'n' => (7, 1),
                        'b' => (8, 1),
                        'r' => (9, 1),
                        'q' => (10, 1),
                        'k' => (11, 1),
                        _ => panic!("Invalid FEN char: {}", ch),
                    };
                    pos.set_piece(sq as u8, piece_index);
                    sq += 1;
                }
            }
        }

        pos.side_to_move = if side_part == "w" { 0 } else { 1 };

        pos.castling_rights = 0;
        if castling_part.contains('K') {
            pos.castling_rights |= 1;
        }
        if castling_part.contains('Q') {
            pos.castling_rights |= 2;
        }
        if castling_part.contains('k') {
            pos.castling_rights |= 4;
        }
        if castling_part.contains('q') {
            pos.castling_rights |= 8;
        }

        pos.ep_square = if ep_part != "-" {
            let file = ep_part.as_bytes()[0] - b'a';
            let rank = ep_part.as_bytes()[1] - b'1';
            rank * 8 + file
        } else {
            64
        };

        pos.halfmove_clock = halfmove_part.parse().unwrap();
        pos.fullmove_number = fullmove_part.parse().unwrap();

        pos
    }

    pub fn debug_print(&self) {
        for rank in (0..8).rev() {
            print!("{} ", rank + 1);
            for file in 0..8 {
                let sq = rank * 8 + file;
                let mut piece_char = '.';
                for (i, bb) in self.pieces.iter().enumerate() {
                    if (bb >> sq) & 1 != 0 {
                        piece_char = match i {
                            0 => 'P',
                            1 => 'N',
                            2 => 'B',
                            3 => 'R',
                            4 => 'Q',
                            5 => 'K',
                            6 => 'p',
                            7 => 'n',
                            8 => 'b',
                            9 => 'r',
                            10 => 'q',
                            11 => 'k',
                            _ => '?',
                        };
                        break;
                    }
                }
                print!("{} ", piece_char);
            }
            println!();
        }
        println!("  a b c d e f g h");
        println!(
            "Side to move: {}",
            if self.side_to_move == 0 {
                "White"
            } else {
                "Black"
            }
        );
        println!("Castling rights: {:04b}", self.castling_rights);
        println!("EP square: {}", self.ep_square);
    }
}

impl Position {
    pub fn apply_move(&self, mv: &Move) -> Position {
        let mut new_pos = self.clone();

        let us = self.side_to_move;
        let them = us ^ 1;

        // 1. Remove moving piece from `from`
        let piece_index = new_pos
            .pieces
            .iter_mut()
            .position(|bb| (*bb >> mv.from) & 1 != 0)
            .map(|idx| {
                new_pos.pieces[idx] &= !(1u64 << mv.from);
                idx
            })
            .expect("No piece on from-square");

        // 2. Handle captures (including en passant)
        if mv.flags & FLAG_CAPTURE != 0 {
            let capture_sq = if mv.flags & FLAG_EN_PASSANT != 0 {
                if us == 0 { mv.to - 8 } else { mv.to + 8 }
            } else {
                mv.to
            };

            if let Some(idx) = new_pos
                .pieces
                .iter_mut()
                .position(|bb| (*bb >> capture_sq) & 1 != 0)
            {
                new_pos.pieces[idx] &= !(1u64 << capture_sq);
            }
        }

        // 3. Handle castling rook move
        if mv.flags & FLAG_KINGSIDE_CASTLE != 0 {
            let (rook_from, rook_to) = if us == 0 { (7, 5) } else { (63, 61) };
            new_pos.move_piece(rook_from, rook_to);
        } else if mv.flags & FLAG_QUEENSIDE_CASTLE != 0 {
            let (rook_from, rook_to) = if us == 0 { (0, 3) } else { (56, 59) };
            new_pos.move_piece(rook_from, rook_to);
        }

        // 4. Place moving piece on `to` (promotion or normal)
        if mv.flags & FLAG_PROMOTION != 0 {
            let promo_index: usize = (us as usize) * 6 + (mv.promotion as usize);
            new_pos.pieces[promo_index] |= bit(mv.to);
        } else {
            new_pos.pieces[piece_index] |= bit(mv.to);
        }

        // 5. Update occupancy
        new_pos.occupancy[0] = new_pos.pieces[0..6].iter().fold(0, |acc, &bb| acc | bb);
        new_pos.occupancy[1] = new_pos.pieces[6..12].iter().fold(0, |acc, &bb| acc | bb);
        new_pos.occupancy[2] = new_pos.occupancy[0] | new_pos.occupancy[1];

        // 6. Update castling rights
        new_pos.update_castling_rights(mv.from, mv.to);

        // 7. Update en passant square
        if self.is_pawn_double_push(piece_index, mv.from, mv.to) {
            new_pos.ep_square = if us == 0 { mv.from + 8 } else { mv.from - 8 };
        } else {
            new_pos.ep_square = 64;
        }

        // 8. Update halfmove clock
        if mv.flags & (FLAG_CAPTURE | FLAG_EN_PASSANT) != 0 || self.is_pawn(piece_index) {
            new_pos.halfmove_clock = 0;
        } else {
            new_pos.halfmove_clock += 1;
        }

        // 9. Update fullmove number
        if us == 1 {
            new_pos.fullmove_number += 1;
        }

        // 10. Switch side
        new_pos.side_to_move = them;

        new_pos
    }

    fn move_piece(&mut self, from: u8, to: u8) {
        for bb in &mut self.pieces {
            if (*bb >> from) & 1 != 0 {
                *bb &= !(1u64 << from);
                *bb |= 1u64 << to;
                break;
            }
        }
    }

    fn update_castling_rights(&mut self, from: u8, to: u8) {
        // Clear rights if king or rook moves/captured
        match from {
            4 => {
                self.castling_rights &= !0b0011;
            } // white king
            60 => {
                self.castling_rights &= !0b1100;
            } // black king
            0 => {
                self.castling_rights &= !0b0010;
            } // white queenside rook
            7 => {
                self.castling_rights &= !0b0001;
            } // white kingside rook
            56 => {
                self.castling_rights &= !0b1000;
            } // black queenside rook
            63 => {
                self.castling_rights &= !0b0100;
            } // black kingside rook
            _ => {}
        }
        match to {
            0 => {
                self.castling_rights &= !0b0010;
            }
            7 => {
                self.castling_rights &= !0b0001;
            }
            56 => {
                self.castling_rights &= !0b1000;
            }
            63 => {
                self.castling_rights &= !0b0100;
            }
            _ => {}
        }
    }

    fn is_pawn(&self, piece_index: usize) -> bool {
        piece_index.is_multiple_of(6)
    }

    fn is_pawn_double_push(&self, piece_index: usize, from: u8, to: u8) -> bool {
        self.is_pawn(piece_index) && (from as i8 - to as i8).abs() == 16
    }
}

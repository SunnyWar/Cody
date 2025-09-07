use crate::core::bitboard::bit;
use crate::core::mov::Move;

// Flags
pub const FLAG_CAPTURE: u8 = 1 << 0;
pub const FLAG_KINGSIDE_CASTLE: u8 = 1 << 1;
pub const FLAG_QUEENSIDE_CASTLE: u8 = 1 << 2;
pub const FLAG_EN_PASSANT: u8 = 1 << 3;
pub const FLAG_PROMOTION: u8 = 1 << 4;

#[derive(Clone, Copy)]
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
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
}

impl Position {
    #[inline]
    pub fn copy_from(&mut self, other: &Position) {
        *self = *other; // struct assignment, no heap
    }

    pub fn all_pieces(&self) -> u64 {
        self.pieces.iter().fold(0u64, |acc, &bb| acc | bb)
    }

    pub fn our_pieces(&self, us: u8) -> u64 {
        let start = if us == 0 { 0 } else { 6 };
        let mut bb = 0u64;
        for i in 0..6 {
            bb |= self.pieces[start + i];
        }
        bb
    }

    pub fn their_pieces(&self, us: u8) -> u64 {
        self.our_pieces(us ^ 1) // flip side: 0→1, 1→0
    }

    #[inline]
    fn set_piece(&mut self, sq: u8, piece_index: usize) {
        let bit = 1u64 << sq;
        self.pieces[piece_index] |= bit;
        let color = piece_index / 6;
        self.occupancy[color] |= bit;
        self.occupancy[2] |= bit;
    }

    pub fn empty() -> Self {
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

    pub fn from_fen(fen: &str) -> Self {
        let mut pos = Position::empty(); // <-- no recursion
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
                '/' => sq -= 16,
                '1'..='8' => sq += ch.to_digit(10).unwrap() as i32,
                _ => {
                    let piece_index = match ch {
                        'P' => 0,
                        'N' => 1,
                        'B' => 2,
                        'R' => 3,
                        'Q' => 4,
                        'K' => 5,
                        'p' => 6,
                        'n' => 7,
                        'b' => 8,
                        'r' => 9,
                        'q' => 10,
                        'k' => 11,
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

    pub fn apply_move_into(&self, mv: &Move, out: &mut Position) {
        // Start as a copy of self — struct assignment, no heap
        *out = *self;

        let us = self.side_to_move;
        let them = us ^ 1;

        // 1. Remove moving piece from `from`
        let piece_index = out
            .pieces
            .iter_mut()
            .position(|bb| (*bb >> mv.from) & 1 != 0)
            .map(|idx| {
                out.pieces[idx] &= !(1u64 << mv.from);
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

            if let Some(idx) = out
                .pieces
                .iter_mut()
                .position(|bb| (*bb >> capture_sq) & 1 != 0)
            {
                out.pieces[idx] &= !(1u64 << capture_sq);
            }
        }

        // 3. Handle castling rook move
        if mv.flags & FLAG_KINGSIDE_CASTLE != 0 {
            let (rook_from, rook_to) = if us == 0 { (7, 5) } else { (63, 61) };
            out.move_piece(rook_from, rook_to);
        } else if mv.flags & FLAG_QUEENSIDE_CASTLE != 0 {
            let (rook_from, rook_to) = if us == 0 { (0, 3) } else { (56, 59) };
            out.move_piece(rook_from, rook_to);
        }

        // 4. Place moving piece on `to` (promotion or normal)
        if mv.flags & FLAG_PROMOTION != 0 {
            let promo_index: usize = (us as usize) * 6 + (mv.promotion as usize);
            out.pieces[promo_index] |= bit(mv.to);
        } else {
            out.pieces[piece_index] |= bit(mv.to);
        }

        // 5. Update occupancy
        out.occupancy[0] = out.pieces[0..6].iter().fold(0, |acc, &bb| acc | bb);
        out.occupancy[1] = out.pieces[6..12].iter().fold(0, |acc, &bb| acc | bb);
        out.occupancy[2] = out.occupancy[0] | out.occupancy[1];

        // 6. Update castling rights
        out.update_castling_rights(mv.from, mv.to);

        // 7. Update en passant square
        if self.is_pawn_double_push(piece_index, mv.from, mv.to) {
            out.ep_square = if us == 0 { mv.from + 8 } else { mv.from - 8 };
        } else {
            out.ep_square = 64;
        }

        // 8. Update halfmove clock
        if mv.flags & (FLAG_CAPTURE | FLAG_EN_PASSANT) != 0 || self.is_pawn(piece_index) {
            out.halfmove_clock = 0;
        } else {
            out.halfmove_clock += 1;
        }

        // 9. Update fullmove number
        if us == 1 {
            out.fullmove_number += 1;
        }

        // 10. Switch side
        out.side_to_move = them;
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

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // 1. Piece placement
        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                let sq = rank * 8 + file;
                let mut piece_char = None;
                for (i, bb) in self.pieces.iter().enumerate() {
                    if (bb >> sq) & 1 != 0 {
                        piece_char = Some(match i {
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
                        });
                        break;
                    }
                }
                if let Some(pc) = piece_char {
                    if empty > 0 {
                        fen.push_str(&empty.to_string());
                        empty = 0;
                    }
                    fen.push(pc);
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                fen.push_str(&empty.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        // 2. Side to move
        fen.push(' ');
        fen.push(if self.side_to_move == 0 { 'w' } else { 'b' });

        // 3. Castling rights
        fen.push(' ');
        let mut castling = String::new();
        if self.castling_rights & 0b0001 != 0 {
            castling.push('K');
        }
        if self.castling_rights & 0b0010 != 0 {
            castling.push('Q');
        }
        if self.castling_rights & 0b0100 != 0 {
            castling.push('k');
        }
        if self.castling_rights & 0b1000 != 0 {
            castling.push('q');
        }
        if castling.is_empty() {
            castling.push('-');
        }
        fen.push_str(&castling);

        // 4. En passant target square
        fen.push(' ');
        if self.ep_square < 64 {
            let file = (self.ep_square % 8) as u8;
            let rank = (self.ep_square / 8) as u8;
            fen.push((b'a' + file) as char);
            fen.push((b'1' + rank) as char);
        } else {
            fen.push('-');
        }

        // 5. Halfmove clock
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());

        // 6. Fullmove number
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }
}

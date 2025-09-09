// src/core/position.rs

use crate::core::bitboard::{
    BISHOP_ATTACKS, BISHOP_MASKS, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, ROOK_ATTACKS,
    ROOK_MASKS, occ_to_index,
};
use crate::core::bitboardmask::{
    BitBoardMask, bishop_attacks_mask, king_attacks_mask, knight_attacks_mask, pawn_attacks_mask,
    queen_attacks_mask, rook_attacks_mask,
};
use crate::core::mov::Move;
use crate::core::piece::{Color, Piece, PieceKind};
use crate::core::piecebitboards::PieceBitboards;
use crate::core::square::Square;

// Flags
pub const FLAG_CAPTURE: u8 = 1 << 0;
pub const FLAG_KINGSIDE_CASTLE: u8 = 1 << 1;
pub const FLAG_QUEENSIDE_CASTLE: u8 = 1 << 2;
pub const FLAG_EN_PASSANT: u8 = 1 << 3;
pub const FLAG_PROMOTION: u8 = 1 << 4;

#[derive(Clone, Copy, PartialEq)]
pub enum OccupancyKind {
    White = 0,
    Black = 1,
    Both = 2,
}

#[derive(Clone, Copy)]
pub struct CastlingRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool,
}

impl CastlingRights {
    pub fn empty() -> Self {
        Self {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false,
        }
    }

    pub fn from_fen(s: &str) -> Self {
        let mut rights = Self::empty();
        if s.contains('K') {
            rights.white_kingside = true;
        }
        if s.contains('Q') {
            rights.white_queenside = true;
        }
        if s.contains('k') {
            rights.black_kingside = true;
        }
        if s.contains('q') {
            rights.black_queenside = true;
        }
        rights
    }

    pub fn to_fen(&self) -> String {
        let mut s = String::new();
        if self.white_kingside {
            s.push('K');
        }
        if self.white_queenside {
            s.push('Q');
        }
        if self.black_kingside {
            s.push('k');
        }
        if self.black_queenside {
            s.push('q');
        }
        if s.is_empty() {
            s.push('-');
        }
        s
    }

    pub fn to_bits(&self) -> u8 {
        (self.white_kingside as u8)
            | ((self.white_queenside as u8) << 1)
            | ((self.black_kingside as u8) << 2)
            | ((self.black_queenside as u8) << 3)
    }

    pub fn clear(&mut self, color: Color, side: bool) {
        match (color, side) {
            (Color::White, true) => self.white_kingside = false,
            (Color::White, false) => self.white_queenside = false,
            (Color::Black, true) => self.black_kingside = false,
            (Color::Black, false) => self.black_queenside = false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Position {
    pub pieces: PieceBitboards,
    pub occupancy: [u64; 3], // Indexed by OccupancyKind
    pub side_to_move: Color,
    pub castling_rights: CastlingRights,
    pub ep_square: Option<u8>,
    pub halfmove_clock: u8,
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
        *self = *other;
    }

    pub fn all_pieces(&self) -> BitBoardMask {
        self.pieces.all()
    }

    pub fn our_pieces(&self, us: Color) -> BitBoardMask {
        let mut acc = BitBoardMask::empty();
        acc |= self
            .pieces
            .get(Piece::from_parts(us, Some(PieceKind::Pawn)));
        acc |= self
            .pieces
            .get(Piece::from_parts(us, Some(PieceKind::Knight)));
        acc |= self
            .pieces
            .get(Piece::from_parts(us, Some(PieceKind::Bishop)));
        acc |= self
            .pieces
            .get(Piece::from_parts(us, Some(PieceKind::Rook)));
        acc |= self
            .pieces
            .get(Piece::from_parts(us, Some(PieceKind::Queen)));
        acc |= self
            .pieces
            .get(Piece::from_parts(us, Some(PieceKind::King)));
        acc
    }

    pub fn their_pieces(&self, us: Color) -> BitBoardMask {
        self.our_pieces(us.opposite())
    }

    #[inline]
    fn set_piece(&mut self, sq: u8, piece: Piece) {
        let mask = BitBoardMask::from_square(sq);

        // Set the piece bitboard
        *self.pieces.get_mut(piece) |= mask;

        // Update occupancy
        let color_idx = match piece.color() {
            Color::White => OccupancyKind::White as usize,
            Color::Black => OccupancyKind::Black as usize,
        };
        self.occupancy[color_idx] |= mask.0;
        self.occupancy[OccupancyKind::Both as usize] |= mask.0;
    }

    pub fn empty() -> Self {
        Self {
            pieces: PieceBitboards::new(),
            occupancy: [0; 3],
            side_to_move: Color::White,
            castling_rights: CastlingRights::empty(),
            ep_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn from_fen(fen: &str) -> Self {
        let mut pos = Position::empty();
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            panic!("Invalid FEN: expected 6 parts");
        }

        let board_part = parts[0];
        let side_part = parts[1];
        let castling_part = parts[2];
        let ep_part = parts[3];
        let halfmove_part = parts[4];
        let fullmove_part = parts[5];

        // Parse board
        let mut sq: i32 = 56; // a8
        for ch in board_part.chars() {
            match ch {
                '/' => sq -= 16,
                '1'..='8' => sq += ch.to_digit(10).unwrap() as i32,
                _ => {
                    let piece =
                        Piece::from_char(ch).unwrap_or_else(|| panic!("Invalid FEN char: {}", ch));
                    pos.set_piece(sq as u8, piece);
                    sq += 1;
                }
            }
        }

        // Side to move
        pos.side_to_move = match side_part {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!("Invalid FEN side: {}", side_part),
        };

        // Castling rights
        pos.castling_rights = CastlingRights::from_fen(castling_part);

        // En passant square
        pos.ep_square = if ep_part != "-" {
            let file = ep_part.as_bytes()[0] - b'a';
            let rank = ep_part.as_bytes()[1] - b'1';
            Some(rank * 8 + file)
        } else {
            None
        };

        // Halfmove / fullmove
        pos.halfmove_clock = halfmove_part.parse().unwrap_or(0);
        pos.fullmove_number = fullmove_part.parse().unwrap_or(1);

        pos
    }

    pub fn debug_print(&self) {
        for rank in (0..8).rev() {
            print!("{} ", rank + 1);
            for file in 0..8 {
                let sq = rank * 8 + file;
                let mut piece_char = '.';
                for (piece, bb) in self.pieces.iter() {
                    if bb.contains_square(sq as u8) {
                        piece_char = piece.to_char();
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
            match self.side_to_move {
                Color::White => "White",
                Color::Black => "Black",
            }
        );

        println!("Castling rights: {}", self.castling_rights.to_fen());
        /* println!(
            "EP square: {}",
            self.ep_square
                .map(|sq| Square::from_index(sq).to_string())
                .unwrap_or("-".to_string())
        ); */
    }

    pub fn apply_move_into(&self, mv: &Move, out: &mut Position) {
        *out = *self;
        let us = self.side_to_move;
        let them = us.opposite();

        let from_mask = BitBoardMask::from_square(mv.from);
        let moving_piece = out
            .pieces
            .iter_mut()
            .find_map(|(piece, bb)| {
                if (*bb & from_mask).is_nonempty() {
                    *bb &= !from_mask;
                    Some(piece)
                } else {
                    None
                }
            })
            .expect("No piece on from-square");

        if mv.flags & FLAG_CAPTURE != 0 {
            let capture_sq = if mv.flags & FLAG_EN_PASSANT != 0 {
                match us {
                    Color::White => mv.to - 8,
                    Color::Black => mv.to + 8,
                }
            } else {
                mv.to
            };

            let cap_mask = BitBoardMask::from_square(capture_sq);
            for kind in [
                PieceKind::Pawn,
                PieceKind::Knight,
                PieceKind::Bishop,
                PieceKind::Rook,
                PieceKind::Queen,
                PieceKind::King,
            ] {
                let p = Piece::from_parts(them, Some(kind));
                let bb = out.pieces.get_mut(p);
                if (*bb & cap_mask).is_nonempty() {
                    *bb &= !cap_mask;
                    break;
                }
            }
        }

        if mv.flags & FLAG_KINGSIDE_CASTLE != 0 {
            let (rook_from, rook_to) = match us {
                Color::White => (7, 5),
                Color::Black => (63, 61),
            };
            let rook = Piece::from_parts(us, Some(PieceKind::Rook));
            let rbb = out.pieces.get_mut(rook);
            *rbb &= !BitBoardMask::from_square(rook_from);
            *rbb |= BitBoardMask::from_square(rook_to);
        } else if mv.flags & FLAG_QUEENSIDE_CASTLE != 0 {
            let (rook_from, rook_to) = match us {
                Color::White => (0, 3),
                Color::Black => (56, 59),
            };
            let rook = Piece::from_parts(us, Some(PieceKind::Rook));
            let rbb = out.pieces.get_mut(rook);
            *rbb &= !BitBoardMask::from_square(rook_from);
            *rbb |= BitBoardMask::from_square(rook_to);
        }

        let to_mask = BitBoardMask::from_square(mv.to);
        if mv.flags & FLAG_PROMOTION != 0 {
            let promo_piece = Piece::from_parts(us, mv.promotion);
            let bb = out.pieces.get_mut(promo_piece);
            *bb |= to_mask;
        } else {
            let bb = out.pieces.get_mut(moving_piece);
            *bb |= to_mask;
        }

        let white_occ = or_color(&out.pieces, Color::White);
        let black_occ = or_color(&out.pieces, Color::Black);
        out.occupancy[OccupancyKind::White as usize] = white_occ.0;
        out.occupancy[OccupancyKind::Black as usize] = black_occ.0;
        out.occupancy[OccupancyKind::Both as usize] = (white_occ | black_occ).0;

        out.update_castling_rights(mv.from, mv.to);

        if is_pawn_double_push(moving_piece, mv.from, mv.to, us) {
            out.ep_square = Some(match us {
                Color::White => mv.from + 8,
                Color::Black => mv.from - 8,
            });
        } else {
            out.ep_square = None;
        }

        let was_capture = mv.flags & (FLAG_CAPTURE | FLAG_EN_PASSANT) != 0;
        let was_pawn_move = moving_piece.kind() == PieceKind::Pawn;
        if was_capture || was_pawn_move {
            out.halfmove_clock = 0;
        } else {
            out.halfmove_clock = out.halfmove_clock.saturating_add(1);
        }

        if us == Color::Black {
            out.fullmove_number = out.fullmove_number.saturating_add(1);
        }

        out.side_to_move = them;
    }

    fn update_castling_rights(&mut self, from: u8, to: u8) {
        match from {
            4 => {
                self.castling_rights.clear(Color::White, true);
                self.castling_rights.clear(Color::White, false);
            } // white king
            60 => {
                self.castling_rights.clear(Color::Black, true);
                self.castling_rights.clear(Color::Black, false);
            } // black king
            0 => self.castling_rights.clear(Color::White, false), // white queenside rook
            7 => self.castling_rights.clear(Color::White, true),  // white kingside rook
            56 => self.castling_rights.clear(Color::Black, false), // black queenside rook
            63 => self.castling_rights.clear(Color::Black, true), // black kingside rook
            _ => {}
        }
        match to {
            0 => self.castling_rights.clear(Color::White, false),
            7 => self.castling_rights.clear(Color::White, true),
            56 => self.castling_rights.clear(Color::Black, false),
            63 => self.castling_rights.clear(Color::Black, true),
            _ => {}
        }
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                let sq = (rank * 8 + file) as u8;
                let mut piece_char = None;

                for (piece, bb) in self.pieces.iter() {
                    if bb.contains_square(sq) {
                        piece_char = Some(piece.to_char());
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

        // Side to move
        fen.push(' ');
        fen.push(match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        });

        // Castling rights
        fen.push(' ');
        fen.push_str(&self.castling_rights.to_fen());

        // En passant square
        fen.push(' ');
        if let Some(sq) = self.ep_square {
            let file = sq % 8;
            let rank = sq / 8;
            fen.push((b'a' + file) as char);
            fen.push((b'1' + rank) as char);
        } else {
            fen.push('-');
        }

        // Halfmove clock
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());

        // Fullmove number
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }

    pub fn parse_uci_move(&self, mv: &str) -> Option<Move> {
        if mv.len() < 4 {
            return None;
        }

        let from_file = mv.as_bytes()[0] as char;
        let from_rank = mv.as_bytes()[1] as char;
        let to_file = mv.as_bytes()[2] as char;
        let to_rank = mv.as_bytes()[3] as char;

        let from_sq = Square::from_coords(from_file, from_rank)?;
        let to_sq = Square::from_coords(to_file, to_rank)?;

        let promo = if mv.len() == 5 {
            PieceKind::from_uci(mv.as_bytes()[4] as char)
        } else {
            None
        };

        self.generate_legal_moves()
            .into_iter()
            .find(|m| m.from() == from_sq && m.to() == to_sq && m.promotion() == promo)
    }

    pub fn generate_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let us = self.side_to_move;
        let them = us.opposite();
        let our_pieces = self.our_pieces(us); // BitBoardMask
        let their_pieces = self.their_pieces(us); // BitBoardMask
        let all_pieces = BitBoardMask(self.occupancy[OccupancyKind::Both as usize]);

        let add_move = |moves: &mut Vec<Move>, from, to, flags, promotion| {
            moves.push(Move {
                from,
                to,
                flags,
                promotion,
            });
        };

        for from in 0..64 {
            if !our_pieces.contains_square(from) {
                continue;
            }

            let piece = self
                .pieces
                .iter()
                .find(|(_, bb)| bb.contains_square(from))
                .map(|(p, _)| p)
                .expect("Piece exists on square");

            match piece.kind() {
                PieceKind::Pawn => {
                    let (forward, second_rank, seventh_rank, ep_rank) = match us {
                        Color::White => (8i8, 8..16, 48..56, 40..48),
                        Color::Black => (-8i8, 48..56, 8..16, 16..24),
                    };

                    let to = (from as i8 + forward) as u8;
                    if to < 64 && !all_pieces.contains_square(to) {
                        if seventh_rank.contains(&from) {
                            for promo in [
                                PieceKind::Queen,
                                PieceKind::Rook,
                                PieceKind::Bishop,
                                PieceKind::Knight,
                            ] {
                                add_move(&mut moves, from, to, FLAG_PROMOTION, Some(promo));
                            }
                        } else {
                            add_move(&mut moves, from, to, 0, None);
                            if second_rank.contains(&from) {
                                let double_to = (from as i8 + 2 * forward) as u8;
                                if !all_pieces.contains_square(double_to) {
                                    add_move(&mut moves, from, double_to, 0, None);
                                }
                            }
                        }
                    }

                    let attacks = pawn_attacks_mask(from, us); // returns BitBoardMask
                    for to in attacks.squares() {
                        if their_pieces.contains_square(to) {
                            if seventh_rank.contains(&from) {
                                for promo in [
                                    PieceKind::Queen,
                                    PieceKind::Rook,
                                    PieceKind::Bishop,
                                    PieceKind::Knight,
                                ] {
                                    add_move(
                                        &mut moves,
                                        from,
                                        to,
                                        FLAG_CAPTURE | FLAG_PROMOTION,
                                        Some(promo),
                                    );
                                }
                            } else {
                                add_move(&mut moves, from, to, FLAG_CAPTURE, None);
                            }
                        }
                        if let Some(ep) = self.ep_square
                            && to == ep
                            && ep_rank.contains(&to)
                        {
                            add_move(&mut moves, from, to, FLAG_CAPTURE | FLAG_EN_PASSANT, None);
                        }
                    }
                }
                PieceKind::Knight => {
                    for to in knight_attacks_mask(from).squares() {
                        let flags = if their_pieces.contains_square(to) {
                            FLAG_CAPTURE
                        } else {
                            0
                        };
                        add_move(&mut moves, from, to, flags, None);
                    }
                }
                PieceKind::Bishop => {
                    for to in bishop_attacks_mask(from, all_pieces).squares() {
                        let flags = if their_pieces.contains_square(to) {
                            FLAG_CAPTURE
                        } else {
                            0
                        };
                        add_move(&mut moves, from, to, flags, None);
                    }
                }
                PieceKind::Rook => {
                    for to in rook_attacks_mask(from, all_pieces).squares() {
                        let flags = if their_pieces.contains_square(to) {
                            FLAG_CAPTURE
                        } else {
                            0
                        };
                        add_move(&mut moves, from, to, flags, None);
                    }
                }
                PieceKind::Queen => {
                    for to in queen_attacks_mask(from, all_pieces).squares() {
                        let flags = if their_pieces.contains_square(to) {
                            FLAG_CAPTURE
                        } else {
                            0
                        };
                        add_move(&mut moves, from, to, flags, None);
                    }
                }
                PieceKind::King => {
                    for to in king_attacks_mask(from).squares() {
                        let flags = if their_pieces.contains_square(to) {
                            FLAG_CAPTURE
                        } else {
                            0
                        };
                        add_move(&mut moves, from, to, flags, None);
                    }
                    // Castling
                    if us == Color::White {
                        if self.castling_rights.white_kingside && (all_pieces.0 & 0x60) == 0 {
                            add_move(&mut moves, 4, 6, FLAG_KINGSIDE_CASTLE, None);
                        }
                        if self.castling_rights.white_queenside && (all_pieces.0 & 0xE) == 0 {
                            add_move(&mut moves, 4, 2, FLAG_QUEENSIDE_CASTLE, None);
                        }
                    } else {
                        if self.castling_rights.black_kingside
                            && (all_pieces.0 & (0x60u64 << 56)) == 0
                        {
                            add_move(&mut moves, 60, 62, FLAG_KINGSIDE_CASTLE, None);
                        }
                        if self.castling_rights.black_queenside
                            && (all_pieces.0 & (0xEu64 << 56)) == 0
                        {
                            add_move(&mut moves, 60, 58, FLAG_QUEENSIDE_CASTLE, None);
                        }
                    }
                }
            }
        }

        // Filter illegal moves
        moves.retain(|mv| {
            let mut new_pos = Position::empty();
            self.apply_move_into(mv, &mut new_pos);
            !new_pos.is_in_check(them)
        });

        moves
    }

    fn is_in_check(&self, side: Color) -> bool {
        let king_square = self
            .pieces
            .get(Piece::from_parts(side, Some(PieceKind::King)))
            .first_square()
            .expect("King not found");
        let attackers = self.attacks_to(king_square, side.opposite());
        !attackers.is_empty()
    }

    fn attacks_to(&self, square: u8, attacker: Color) -> BitBoardMask {
        let mut attacks = BitBoardMask::empty();
        let pawns = pawn_attacks(square, attacker.opposite());
        attacks |= pawns
            & self
                .pieces
                .get(Piece::from_parts(attacker, Some(PieceKind::Pawn)));
        let knights = knight_attacks(square);
        attacks |= knights
            & self
                .pieces
                .get(Piece::from_parts(attacker, Some(PieceKind::Knight)));
        let bishops = bishop_attacks(
            square,
            BitBoardMask(self.occupancy[OccupancyKind::Both as usize]),
        );

        attacks |= bishops
            & self
                .pieces
                .get(Piece::from_parts(attacker, Some(PieceKind::Bishop)));
        let rooks = rook_attacks(
            square,
            BitBoardMask(self.occupancy[OccupancyKind::Both as usize]),
        );
        attacks |= rooks
            & self
                .pieces
                .get(Piece::from_parts(attacker, Some(PieceKind::Rook)));
        let queens = queen_attacks(
            square,
            BitBoardMask(self.occupancy[OccupancyKind::Both as usize]),
        );
        attacks |= queens
            & self
                .pieces
                .get(Piece::from_parts(attacker, Some(PieceKind::Queen)));
        let kings = king_attacks(square);
        attacks |= kings
            & self
                .pieces
                .get(Piece::from_parts(attacker, Some(PieceKind::King)));
        attacks
    }
}

#[inline]
fn or_color(pieces: &PieceBitboards, c: Color) -> BitBoardMask {
    let mut acc = BitBoardMask::empty();
    acc |= pieces.get(Piece::from_parts(c, Some(PieceKind::Pawn)));
    acc |= pieces.get(Piece::from_parts(c, Some(PieceKind::Knight)));
    acc |= pieces.get(Piece::from_parts(c, Some(PieceKind::Bishop)));
    acc |= pieces.get(Piece::from_parts(c, Some(PieceKind::Rook)));
    acc |= pieces.get(Piece::from_parts(c, Some(PieceKind::Queen)));
    acc |= pieces.get(Piece::from_parts(c, Some(PieceKind::King)));
    acc
}

#[inline]
fn is_pawn_double_push(piece: Piece, from: u8, to: u8, side: Color) -> bool {
    if piece.kind() != PieceKind::Pawn {
        return false;
    }
    match side {
        Color::White => (from / 8 == 1) && (to == from + 16),
        Color::Black => (from / 8 == 6) && (to == from - 16),
    }
}

#[inline]
fn knight_attacks(square: u8) -> BitBoardMask {
    BitBoardMask(KNIGHT_ATTACKS[square as usize])
}

#[inline]
fn king_attacks(square: u8) -> BitBoardMask {
    BitBoardMask(KING_ATTACKS[square as usize])
}

#[inline]
fn pawn_attacks(square: u8, color: Color) -> BitBoardMask {
    BitBoardMask(PAWN_ATTACKS[color as usize][square as usize])
}

#[inline]
fn bishop_attacks(square: u8, occupied: BitBoardMask) -> BitBoardMask {
    let sq = square as usize;
    let mask = BISHOP_MASKS[sq];
    let index = occ_to_index(occupied.0 & mask, mask);
    BitBoardMask(BISHOP_ATTACKS[sq][index])
}

#[inline]
fn rook_attacks(square: u8, occupied: BitBoardMask) -> BitBoardMask {
    let sq = square as usize;
    let mask = ROOK_MASKS[sq];
    let index = occ_to_index(occupied.0 & mask, mask);
    BitBoardMask(ROOK_ATTACKS[sq][index])
}

#[inline]
fn queen_attacks(square: u8, occupied: BitBoardMask) -> BitBoardMask {
    bishop_attacks(square, occupied) | rook_attacks(square, occupied)
}

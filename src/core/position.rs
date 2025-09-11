// src/core/position.rs

use crate::core::bitboardmask::BitBoardMask;
use crate::core::castling::CastlingRights;
use crate::core::mov::Move;
use crate::core::occupancy::{OccupancyKind, OccupancyMap};
use crate::core::piece::{Color, Piece, PieceKind};
use crate::core::piecebitboards::PieceBitboards;
use crate::core::square::Square;
use crate::search::movegen::generate_moves;

// Flags
const FLAG_CAPTURE: u8 = 1 << 0;
const FLAG_KINGSIDE_CASTLE: u8 = 1 << 1;
const FLAG_QUEENSIDE_CASTLE: u8 = 1 << 2;
const FLAG_EN_PASSANT: u8 = 1 << 3;
const FLAG_PROMOTION: u8 = 1 << 4;

pub struct MoveGenContext {
    pub us: Color,
    pub not_ours: BitBoardMask,
    pub occupancy: BitBoardMask,
}

#[derive(Clone, Copy)]
pub struct Position {
    pub pieces: PieceBitboards,
    pub occupancyupancy: OccupancyMap,
    pub side_to_move: Color,
    pub castling_rights: CastlingRights,
    pub ep_square: Option<Square>,
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
    fn set_piece(&mut self, sq: Square, piece: Piece) {
        let bit = BitBoardMask::from_square(sq);
        *self.pieces.get_mut(piece) |= bit;

        let color_occupancy = match piece.color() {
            Color::White => OccupancyKind::White,
            Color::Black => OccupancyKind::Black,
        };

        self.occupancyupancy.or_in(color_occupancy, bit);
        self.occupancyupancy.or_in(OccupancyKind::Both, bit);
    }

    fn empty() -> Self {
        Self {
            pieces: PieceBitboards::new(),
            occupancyupancy: OccupancyMap::new(),
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
        let mut rank: u8 = 7;
        let mut file: u8 = 0;
        for ch in board_part.chars() {
            match ch {
                '/' => {
                    rank = rank.saturating_sub(1);
                    file = 0;
                }
                '1'..='8' => {
                    file += ch.to_digit(10).unwrap() as u8;
                }
                _ => {
                    let piece =
                        Piece::from_char(ch).unwrap_or_else(|| panic!("Invalid FEN char: {}", ch));
                    let square = Square::from_rank_file(rank, file)
                        .unwrap_or_else(|| panic!("Invalid square: rank {}, file {}", rank, file));
                    pos.set_piece(square, piece);
                    file += 1;
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
            Square::from_rank_file(rank, file)
        } else {
            None
        };

        // Halfmove / fullmove
        pos.halfmove_clock = halfmove_part.parse().unwrap_or(0);
        pos.fullmove_number = fullmove_part.parse().unwrap_or(1);

        pos
    }

    /* pub fn debug_print(&self) {
        for rank in (0..8).rev() {
            print!("{} ", rank + 1);
            for file in 0..8 {
                let square = Square::from_rank_file(rank as u8, file as u8); // assuming this exists
                let mut piece_char = '.';
                for (piece, bb) in self.pieces.iter() {
                    if bb.contains_square(square) {
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

        /*
        println!(
            "EP square: {}",
            self.ep_square
                .map(|sq| sq.to_string()) // assuming ep_square is Option<Square>
                .unwrap_or("-".to_string())
        );
        */
    } */

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

        // Handle captures
        if mv.flags & FLAG_CAPTURE != 0 {
            let capture_sq = if mv.flags & FLAG_EN_PASSANT != 0 {
                match us {
                    Color::White => mv.to.backward(1).expect("Invalid EP capture square"),
                    Color::Black => mv.to.forward(1).expect("Invalid EP capture square"),
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

        // Handle castling
        if mv.flags & FLAG_KINGSIDE_CASTLE != 0 {
            let (rook_from, rook_to) = match us {
                Color::White => (Square::H1, Square::F1),
                Color::Black => (Square::H8, Square::F8),
            };
            let rook = Piece::from_parts(us, Some(PieceKind::Rook));
            let rbb = out.pieces.get_mut(rook);
            *rbb &= !BitBoardMask::from_square(rook_from);
            *rbb |= BitBoardMask::from_square(rook_to);
        } else if mv.flags & FLAG_QUEENSIDE_CASTLE != 0 {
            let (rook_from, rook_to) = match us {
                Color::White => (Square::A1, Square::D1),
                Color::Black => (Square::A8, Square::D8),
            };
            let rook = Piece::from_parts(us, Some(PieceKind::Rook));
            let rbb = out.pieces.get_mut(rook);
            *rbb &= !BitBoardMask::from_square(rook_from);
            *rbb |= BitBoardMask::from_square(rook_to);
        }

        // Handle promotion or normal move
        let to_mask = BitBoardMask::from_square(mv.to);
        if mv.flags & FLAG_PROMOTION != 0 {
            let promo_piece = Piece::from_parts(us, mv.promotion);
            let bb = out.pieces.get_mut(promo_piece);
            *bb |= to_mask;
        } else {
            let bb = out.pieces.get_mut(moving_piece);
            *bb |= to_mask;
        }

        // Update occupancyupancy
        let white_occupancy = or_color(&out.pieces, Color::White);
        let black_occupancy = or_color(&out.pieces, Color::Black);
        out.occupancyupancy[OccupancyKind::White] = white_occupancy;
        out.occupancyupancy[OccupancyKind::Black] = black_occupancy;
        out.occupancyupancy[OccupancyKind::Both] = white_occupancy | black_occupancy;

        // Update castling rights
        out.update_castling_rights(mv.from, mv.to);

        // Update en passant square
        out.ep_square = if is_pawn_double_push(moving_piece, mv.from, mv.to, us) {
            match us {
                Color::White => mv.from.forward(1),
                Color::Black => mv.from.backward(1),
            }
        } else {
            None
        };

        // Update halfmove clock
        let was_capture = mv.flags & (FLAG_CAPTURE | FLAG_EN_PASSANT) != 0;
        let was_pawn_move = moving_piece.kind() == PieceKind::Pawn;
        out.halfmove_clock = if was_capture || was_pawn_move {
            0
        } else {
            out.halfmove_clock.saturating_add(1)
        };

        // Update fullmove number
        if us == Color::Black {
            out.fullmove_number = out.fullmove_number.saturating_add(1);
        }

        // Switch side to move
        out.side_to_move = them;
    }

    fn update_castling_rights(&mut self, from: Square, to: Square) {
        use Square::*;

        match from {
            E1 => {
                self.castling_rights.clear(Color::White, true);
                self.castling_rights.clear(Color::White, false);
            }
            E8 => {
                self.castling_rights.clear(Color::Black, true);
                self.castling_rights.clear(Color::Black, false);
            }
            A1 => self.castling_rights.clear(Color::White, false),
            H1 => self.castling_rights.clear(Color::White, true),
            A8 => self.castling_rights.clear(Color::Black, false),
            H8 => self.castling_rights.clear(Color::Black, true),
            _ => {}
        }

        match to {
            A1 => self.castling_rights.clear(Color::White, false),
            H1 => self.castling_rights.clear(Color::White, true),
            A8 => self.castling_rights.clear(Color::Black, false),
            H8 => self.castling_rights.clear(Color::Black, true),
            _ => {}
        }
    }

    fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                let square = Square::from_rank_file(rank, file).unwrap();
                let mut piece_char = None;

                for (piece, bb) in self.pieces.iter() {
                    if bb.contains_square(square) {
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
            fen.push(sq.file_char());
            fen.push(sq.rank_char());
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

        // should use: pub trait MoveGenerator {
        //    fn generate_moves(&self, pos: &Position) -> Vec<Move>;
        //    fn in_check(&self, pos: &Position) -> bool;

        generate_moves(self)
            .into_iter()
            .find(|m| m.from() == from_sq && m.to() == to_sq && m.promotion() == promo)
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
fn is_pawn_double_push(piece: Piece, from: Square, to: Square, side: Color) -> bool {
    if piece.kind() != PieceKind::Pawn {
        return false;
    }

    match side {
        Color::White => from.rank() == 1 && from.forward(2).map_or(false, |target| target == to),
        Color::Black => from.rank() == 6 && from.backward(2).map_or(false, |target| target == to),
    }
}

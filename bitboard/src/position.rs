// bitboard/src/position.rs

use crate::BitBoardMask;
use crate::Square;
use crate::attack::BoardState;
use crate::attack::PieceSet;
use crate::attack::is_square_attacked;
use crate::castling::CastlingRights;
use crate::mov::ChessMove;
use crate::mov::MoveType;
use crate::movegen::generate_legal_moves;
use crate::movegen::generate_pseudo_moves;
use crate::occupancy::OccupancyKind;
use crate::occupancy::OccupancyMap;
use crate::piece::Color;
use crate::piece::Piece;
use crate::piece::PieceKind;
use crate::piece::{self};
use crate::piecebitboards::PieceBitboards;

pub struct MoveGenContext {
    pub us: Color,
    pub not_ours: BitBoardMask,
    pub occupancy: BitBoardMask,
}

#[derive(Clone, Copy, Debug)]
#[repr(align(64))]
pub struct Position {
    pub pieces: PieceBitboards,
    pub piece_on: [Piece; 64],
    pub occupancy: OccupancyMap,
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
    /// Fast, flat copy of a complete `Position`.
    ///
    /// Uses `core::ptr::copy_nonoverlapping` for efficient bulk memory copying.
    /// This explicit operation allows better compiler optimization without
    /// relying on cross-crate inlining, while executing millions of times per
    /// second in the hot search path.
    pub fn copy_from(&mut self, other: &Position) {
        // Safety: self and other are both valid, properly aligned Position structs.
        // They may not overlap since self is &mut and other is &.
        unsafe {
            core::ptr::copy_nonoverlapping(
                other as *const Position as *const u8,
                self as *mut Position as *mut u8,
                core::mem::size_of::<Position>(),
            );
        }
    }

    // `all_pieces` is invoked in every move-generation call. Mark it
    // engine) can benefit from cross-crate inlining without relying on LTO.
    pub fn all_pieces(&self) -> BitBoardMask {
        // Direct accessor bypasses Index trait overhead and uses cached union.
        self.occupancy.get_both()
    }

    pub fn our_pieces(&self, us: Color) -> BitBoardMask {
        // Direct accessor bypasses lookup table and Index trait overhead.
        // Uses Color discriminants (White=0, Black=1) directly as array indices.
        self.occupancy.get_by_color(us)
    }

    pub fn can_castle_kingside(&self, color: Color) -> bool {
        let (king_sq, f_sq, g_sq) = match color {
            Color::White => (Square::E1, Square::F1, Square::G1),
            Color::Black => (Square::E8, Square::F8, Square::G8),
        };

        if !self.castling_rights.kingside(color) {
            return false;
        }

        let occ = self.occupancy[OccupancyKind::Both];
        if !(occ & f_sq.bitboard()).is_empty() || !(occ & g_sq.bitboard()).is_empty() {
            return false;
        }

        let board_state = self.to_board_state();
        !is_square_attacked(king_sq, color.opposite(), &board_state)
            && !is_square_attacked(f_sq, color.opposite(), &board_state)
            && !is_square_attacked(g_sq, color.opposite(), &board_state)
    }

    pub fn can_castle_queenside(&self, color: Color) -> bool {
        let (king_sq, d_sq, c_sq, b_sq) = match color {
            Color::White => (Square::E1, Square::D1, Square::C1, Square::B1),
            Color::Black => (Square::E8, Square::D8, Square::C8, Square::B8),
        };

        if !self.castling_rights.queenside(color) {
            return false;
        }

        let occ = self.occupancy[OccupancyKind::Both];
        if !(occ & d_sq.bitboard()).is_empty()
            || !(occ & c_sq.bitboard()).is_empty()
            || !(occ & b_sq.bitboard()).is_empty()
        {
            return false;
        }

        let board_state = self.to_board_state();
        !is_square_attacked(king_sq, color.opposite(), &board_state)
            && !is_square_attacked(d_sq, color.opposite(), &board_state)
            && !is_square_attacked(c_sq, color.opposite(), &board_state)
    }

    pub fn their_pieces(&self, us: Color) -> BitBoardMask {
        self.our_pieces(us.opposite())
    }

    fn set_piece(&mut self, sq: Square, piece: Piece) {
        let bit = BitBoardMask::from_square(sq);
        *self.pieces.get_mut(piece) |= bit;
        self.piece_on[sq.index()] = piece;

        let color_occupancy = match piece.color() {
            Color::White => OccupancyKind::White,
            Color::Black => OccupancyKind::Black,
        };

        self.occupancy.or_in(color_occupancy, bit);
        self.occupancy.or_in(OccupancyKind::Both, bit);
    }

    fn empty() -> Self {
        Self {
            pieces: PieceBitboards::new(),
            piece_on: [Piece::None; 64],
            occupancy: OccupancyMap::new(),
            side_to_move: Color::White,
            castling_rights: CastlingRights::empty(),
            ep_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        let piece = self.piece_on[sq.index()];
        if piece == Piece::None {
            None
        } else {
            Some(piece)
        }
    }

    /// Direct piece accessor returning Piece (which may be Piece::None).
    /// Eliminates Option wrapping overhead by returning the raw Piece value.
    /// Preferred in hot paths where None is explicitly checked (SEE,
    /// quiescence).
    ///
    /// Returns Piece::None if square is empty.
    #[inline(always)]
    pub fn piece_at_square(&self, sq: Square) -> Piece {
        // Safety: Square::index() is always in-bounds for [0..64].
        unsafe { *self.piece_on.get_unchecked(sq.index()) }
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

    pub fn apply_move_into(&self, mv: &ChessMove, out: &mut Position) {
        // Copy entire position in one memcpy (Position is Copy)
        // Much faster than field-by-field assignment
        *out = *self;

        let us = self.side_to_move;
        let them = us.opposite();

        let from_mask = BitBoardMask::from_square(mv.from);

        // Remove the moving piece from its original square.
        // Fast O(1) lookup via square-indexed cache.
        let moving_piece = self.piece_on[mv.from.index()];
        debug_assert!(moving_piece != Piece::None, "No piece on from-square");
        // Only clear the moving piece's bitboard in out
        *out.pieces.get_mut(moving_piece) &= !from_mask;
        out.piece_on[mv.from.index()] = Piece::None;
        // ...existing code...

        // Handle captures (including en passant)
        let capture_sq = match mv.move_type {
            MoveType::EnPassant => match us {
                Color::White => mv.to.backward(1).expect("Invalid EP capture square"),
                Color::Black => mv.to.forward(1).expect("Invalid EP capture square"),
            },
            MoveType::Capture => {
                // ...removed debug output...
                mv.to
            }
            _ => mv.to,
        };

        let target_piece = out.piece_on[mv.to.index()];
        let is_promo_capture = matches!(mv.move_type, MoveType::Promotion(_))
            && target_piece != Piece::None
            && target_piece.color() == them;

        if matches!(mv.move_type, MoveType::Capture | MoveType::EnPassant) || is_promo_capture {
            let cap_mask = BitBoardMask::from_square(capture_sq);
            let captured_piece = out.piece_on[capture_sq.index()];
            if captured_piece != Piece::None {
                // ...removed debug output...
                *out.pieces.get_mut(captured_piece) &= !cap_mask;
                out.piece_on[capture_sq.index()] = Piece::None;
            }
        }

        // Handle castling
        match mv.move_type {
            MoveType::CastleKingside => {
                let (rook_from, rook_to) = match us {
                    Color::White => (Square::H1, Square::F1),
                    Color::Black => (Square::H8, Square::F8),
                };
                let rook = Piece::from_parts(us, Some(PieceKind::Rook));
                let rbb = out.pieces.get_mut(rook);
                *rbb &= !BitBoardMask::from_square(rook_from);
                *rbb |= BitBoardMask::from_square(rook_to);
                out.piece_on[rook_from.index()] = Piece::None;
                out.piece_on[rook_to.index()] = rook;
            }
            MoveType::CastleQueenside => {
                let (rook_from, rook_to) = match us {
                    Color::White => (Square::A1, Square::D1),
                    Color::Black => (Square::A8, Square::D8),
                };
                let rook = Piece::from_parts(us, Some(PieceKind::Rook));
                let rbb = out.pieces.get_mut(rook);
                *rbb &= !BitBoardMask::from_square(rook_from);
                *rbb |= BitBoardMask::from_square(rook_to);
                out.piece_on[rook_from.index()] = Piece::None;
                out.piece_on[rook_to.index()] = rook;
            }
            _ => {}
        }

        // Handle promotion or normal move
        let to_mask = BitBoardMask::from_square(mv.to);
        let final_piece = match mv.move_type {
            MoveType::Promotion(kind) => {
                // ...removed debug output...
                Piece::from_parts(us, Some(kind))
            }
            _ => moving_piece,
        };
        let bb = out.pieces.get_mut(final_piece);
        *bb |= to_mask;
        out.piece_on[mv.to.index()] = final_piece;
        // ...removed debug output...

        // Update occupancy
        let white_occupancy = or_color(&out.pieces, Color::White);
        let black_occupancy = or_color(&out.pieces, Color::Black);
        out.occupancy[OccupancyKind::White] = white_occupancy;
        out.occupancy[OccupancyKind::Black] = black_occupancy;
        out.occupancy[OccupancyKind::Both] = white_occupancy | black_occupancy;

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
        let was_capture =
            matches!(mv.move_type, MoveType::Capture | MoveType::EnPassant) || is_promo_capture;
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
        fn clear_for_square(rights: &mut CastlingRights, sq: Square) {
            use Square::*;
            match sq {
                E1 => {
                    rights.clear(Color::White, true);
                    rights.clear(Color::White, false);
                }
                E8 => {
                    rights.clear(Color::Black, true);
                    rights.clear(Color::Black, false);
                }
                A1 => rights.clear(Color::White, false),
                H1 => rights.clear(Color::White, true),
                A8 => rights.clear(Color::Black, false),
                H8 => rights.clear(Color::Black, true),
                _ => {}
            }
        }

        clear_for_square(&mut self.castling_rights, from);
        clear_for_square(&mut self.castling_rights, to);
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                let square = Square::from_rank_file(rank, file).unwrap();
                let piece_char = self.piece_at(square).map(Piece::to_char);

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

    pub fn parse_uci_move(&self, mv: &str) -> Option<ChessMove> {
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

        // Search through LEGAL moves only (not pseudo-legal) to ensure move is valid
        let moves = generate_legal_moves(self);
        moves
            .into_iter()
            .find(|m| m.from() == from_sq && m.to() == to_sq && m.promotion() == promo)
    }

    /// Parse UCI move from a trusted external source (GUI / protocol stream).
    ///
    /// This first tries strict legal parsing, then falls back to pseudo-legal
    /// parsing with legality verification to reject moves that leave the king  
    /// in check. This prevents desync while remaining robust to minor parsing  
    /// discrepancies.
    pub fn parse_uci_move_trusted(&self, mv: &str) -> Option<ChessMove> {
        // First try strict legal move parsing
        if let Some(legal_mv) = self.parse_uci_move(mv) {
            return Some(legal_mv);
        }

        // Fallback: try pseudo-legal matching with validation
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

        let candidate = generate_pseudo_moves(self)
            .into_iter()
            .find(|m| m.from() == from_sq && m.to() == to_sq && m.promotion() == promo)?;

        // CRITICAL: Verify the pseudo-legal move doesn't leave our king in check.
        // This prevents desync from accepting moves that are mechanically possible
        // but illegal (king in check), which should never occur in valid UCI streams.
        let mut tmp_pos = Position::default();
        self.apply_move_into(&candidate, &mut tmp_pos);

        // Find the king square of the side that just moved
        let moving_side = self.side_to_move;
        let king_piece = Piece::from_parts(moving_side, Some(PieceKind::King));
        let king_bb = tmp_pos.pieces.get(king_piece);
        let king_sq = king_bb.squares().next()?;

        // Check if the king is now attacked by the opponent
        let board_state = tmp_pos.to_board_state();
        if is_square_attacked(king_sq, moving_side.opposite(), &board_state) {
            // Reject: this move would leave our king in check (illegal)
            return None;
        }

        Some(candidate)
    }

    pub fn to_board_state(&self) -> BoardState {
        BoardState {
            occupancy: self.occupancy[crate::occupancy::OccupancyKind::Both],
            white_pieces: PieceSet {
                pawns: self.pieces.get(piece::Piece::WhitePawn),
                knights: self.pieces.get(piece::Piece::WhiteKnight),
                bishops: self.pieces.get(piece::Piece::WhiteBishop),
                rooks: self.pieces.get(piece::Piece::WhiteRook),
                queens: self.pieces.get(piece::Piece::WhiteQueen),
                king: self.pieces.get(piece::Piece::WhiteKing),
            },
            black_pieces: PieceSet {
                pawns: self.pieces.get(piece::Piece::BlackPawn),
                knights: self.pieces.get(piece::Piece::BlackKnight),
                bishops: self.pieces.get(piece::Piece::BlackBishop),
                rooks: self.pieces.get(piece::Piece::BlackRook),
                queens: self.pieces.get(piece::Piece::BlackQueen),
                king: self.pieces.get(piece::Piece::BlackKing),
            },
        }
    }

    /// Compute a 64-bit Zobrist hash for this position.
    pub fn zobrist_hash(&self) -> u64 {
        crate::zobrist::compute_zobrist(self)
    }
}

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

fn is_pawn_double_push(piece: Piece, from: Square, to: Square, side: Color) -> bool {
    if piece.kind() != PieceKind::Pawn {
        return false;
    }

    match side {
        Color::White => from.rank() == 1 && (from.forward(2) == Some(to)),
        Color::Black => from.rank() == 6 && (from.backward(2) == Some(to)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fen_roundtrip() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = Position::from_fen(fen);
        assert_eq!(pos.to_fen(), fen);
    }
}

// bitboard/src/attacks.rs
use crate::BitBoardMask;
use crate::Square;
use crate::bitboard::ANTIDIAGONAL_MASKS;
use crate::bitboard::BISHOP_MASKS;
use crate::bitboard::DIAGONAL_MASKS;
use crate::bitboard::PAWN_ATTACKS;
use crate::bitboard::ROOK_MASKS;
use crate::bitboard::occupancy_to_index;
use crate::piece::Color;
use crate::tables::bishop_attack::BISHOP_ATTACKS;
use crate::tables::file_masks::FILE_MASKS;
use crate::tables::king_attack::KING_ATTACKS;
use crate::tables::knight_attack::KNIGHT_ATTACKS;
use crate::tables::rank_masks::RANK_MASKS;
use crate::tables::rook_attack::ROOK_ATTACKS;

/// Represents a board position for attack calculations
pub struct BoardState {
    pub occupancy: BitBoardMask,
    pub white_pieces: PieceSet,
    pub black_pieces: PieceSet,
}

pub struct PieceSet {
    pub pawns: BitBoardMask,
    pub knights: BitBoardMask,
    pub bishops: BitBoardMask,
    pub rooks: BitBoardMask,
    pub queens: BitBoardMask,
    pub king: BitBoardMask,
}

/// Check if a square is attacked by the given color
pub fn is_square_attacked(square: Square, by_color: Color, board: &BoardState) -> bool {
    let sq_index = square.index();
    let king_color_mask = square.color_mask();

    let attacking_pieces = match by_color {
        Color::White => &board.white_pieces,
        Color::Black => &board.black_pieces,
    };

    // Check knight attacks ─ knights always land on the opposite-coloured square,
    // so we mask once and perform a single population test.
    // SAFETY: sq_index is guaranteed to be 0..64 (from Square::index())
    let knight_attacks = unsafe { *KNIGHT_ATTACKS.get_unchecked(sq_index) };
    if (knight_attacks & attacking_pieces.knights & !king_color_mask).is_nonempty() {
        return true;
    }

    // Check pawn attacks (same color squares only)
    let opponent_color = by_color.opposite();
    let pawn_like = attacking_pieces.pawns & king_color_mask;
    if !pawn_like.is_empty() {
        // SAFETY: opponent_color is 0 or 1, sq_index is 0..64
        let pawn_attacks = unsafe {
            *PAWN_ATTACKS
                .get_unchecked(opponent_color as usize)
                .get_unchecked(sq_index)
        };
        if (pawn_attacks & pawn_like).is_nonempty() {
            return true;
        }
    }

    // Check king attacks (same color squares only)
    let king_like = attacking_pieces.king & king_color_mask;
    if !king_like.is_empty() {
        // SAFETY: sq_index is guaranteed to be 0..64
        let king_attacks = unsafe { *KING_ATTACKS.get_unchecked(sq_index) };
        if (king_attacks & king_like).is_nonempty() {
            return true;
        }
    }

    // Check rook/queen attacks (same rank/file)
    let rank = sq_index / 8;
    let file = sq_index % 8;
    // SAFETY: rank and file are guaranteed to be 0..8
    let rank_mask = unsafe { *RANK_MASKS.get_unchecked(rank) };
    let file_mask = unsafe { *FILE_MASKS.get_unchecked(file) };
    let rook_like = (attacking_pieces.rooks | attacking_pieces.queens) & (rank_mask | file_mask);
    if !rook_like.is_empty() {
        // SAFETY: sq_index is guaranteed to be 0..64
        let rmask = unsafe { *ROOK_MASKS.get_unchecked(sq_index) };
        let rindex = occupancy_to_index(board.occupancy, rmask);
        // SAFETY: rindex is guaranteed valid by occupancy_to_index
        let rook_attacks = unsafe { *ROOK_ATTACKS.get_unchecked(sq_index).get_unchecked(rindex) };
        if (rook_attacks & rook_like).is_nonempty() {
            return true;
        }
    }

    // Check bishop/queen attacks (same color + diagonal)
    // SAFETY: sq_index is guaranteed to be 0..64
    let diag_mask = unsafe { *DIAGONAL_MASKS.get_unchecked(sq_index) };
    let antidiag_mask = unsafe { *ANTIDIAGONAL_MASKS.get_unchecked(sq_index) };
    let bishop_like = (attacking_pieces.bishops | attacking_pieces.queens)
        & king_color_mask
        & (diag_mask | antidiag_mask);
    if !bishop_like.is_empty() {
        // SAFETY: sq_index is guaranteed to be 0..64
        let bmask = unsafe { *BISHOP_MASKS.get_unchecked(sq_index) };
        let bindex = occupancy_to_index(board.occupancy, bmask);
        // SAFETY: bindex is guaranteed valid by occupancy_to_index
        let bishop_attacks =
            unsafe { *BISHOP_ATTACKS.get_unchecked(sq_index).get_unchecked(bindex) };
        if (bishop_attacks & bishop_like).is_nonempty() {
            return true;
        }
    }

    false
}

/// Check if the king of the given color is in check
pub fn is_king_in_check(king_color: Color, board: &BoardState) -> bool {
    // Fetch the king bitboard for the required colour.
    let king_bb = match king_color {
        Color::White => board.white_pieces.king,
        Color::Black => board.black_pieces.king,
    };

    // No king on board means illegal position; treat as not in check.
    let king_bits = king_bb.0;
    if king_bits == 0 {
        return false;
    }

    // SAFETY: king_bits has at least one set bit, so trailing_zeros is in 0..63.
    // Square is repr(u8) over 0..63 and the king bitboard has a single legal
    // square.
    let king_square: Square = unsafe {
        core::mem::transmute::<u8, Square>(
            crate::intrinsics::trailing_zeros_nonzero(king_bits) as u8
        )
    };

    is_square_attacked(king_square, king_color.opposite(), board)
}

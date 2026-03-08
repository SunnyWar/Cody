// bitboard/src/attacks.rs
use crate::BitBoardMask;
use crate::Square;
use crate::bitboard::ANTIDIAGONAL_MASKS;
use crate::bitboard::BISHOP_MASKS;
use crate::bitboard::DIAGONAL_MASKS;
use crate::bitboard::PAWN_ATTACKS;
use crate::bitboard::ROOK_MASKS;
#[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
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
    if (KNIGHT_ATTACKS[sq_index] & attacking_pieces.knights & !king_color_mask).is_nonempty() {
        return true;
    }

    // Check pawn attacks (same color squares only)
    let opponent_color = by_color.opposite();
    let pawn_like = attacking_pieces.pawns & king_color_mask;
    if !pawn_like.is_empty()
        && (PAWN_ATTACKS[opponent_color as usize][sq_index] & pawn_like).is_nonempty()
    {
        return true;
    }

    // Check king attacks (same color squares only)
    let king_like = attacking_pieces.king & king_color_mask;
    if !king_like.is_empty() && (KING_ATTACKS[sq_index] & king_like).is_nonempty() {
        return true;
    }

    // Check rook/queen attacks (same rank/file)
    let rank = sq_index / 8;
    let file = sq_index % 8;
    let rook_like =
        (attacking_pieces.rooks | attacking_pieces.queens) & (RANK_MASKS[rank] | FILE_MASKS[file]);
    if !rook_like.is_empty() {
        let rmask = ROOK_MASKS[sq_index];
        let rindex = occupancy_to_index(board.occupancy, rmask);
        if (ROOK_ATTACKS[sq_index][rindex] & rook_like).is_nonempty() {
            return true;
        }
    }

    // Check bishop/queen attacks (same color + diagonal)
    let bishop_like = (attacking_pieces.bishops | attacking_pieces.queens)
        & king_color_mask
        & (DIAGONAL_MASKS[sq_index] | ANTIDIAGONAL_MASKS[sq_index]);
    if !bishop_like.is_empty() {
        let bmask = BISHOP_MASKS[sq_index];
        let bindex = occupancy_to_index(board.occupancy, bmask);
        if (BISHOP_ATTACKS[sq_index][bindex] & bishop_like).is_nonempty() {
            return true;
        }
    }

    false
}

/// Check if the king of the given color is in check
pub fn is_king_in_check(king_color: Color, board: &BoardState) -> bool {
    // Fetch the king bitboard for the required colour.
    // `matches!` avoids generating the larger `match` construct.
    let king_bb = if matches!(king_color, Color::White) {
        board.white_pieces.king
    } else {
        board.black_pieces.king
    };

    // No king on the board (illegal position) – cannot be in check.
    let bb_bits = king_bb.0;
    if bb_bits == 0 {
        return false;
    }

    // SAFETY: `bb_bits` has exactly one bit set (the king), so the index
    // produced by `trailing_zeros` is guaranteed to be in 0..64.
    let king_square: Square = unsafe {
        core::mem::transmute::<u8, Square>(crate::intrinsics::trailing_zeros(bb_bits) as u8)
    };

    is_square_attacked(king_square, king_color.opposite(), board)
}

use crate::Square;
use crate::bitboard::bishop_attacks_from;
use crate::bitboard::king_attacks;
use crate::bitboard::knight_attacks;
use crate::bitboard::pawn_attacks_to;
use crate::bitboard::rook_attacks_from;
use crate::mov::ChessMove;
use crate::occupancy::OccupancyKind;
use crate::piece::Color;
use crate::piece::Piece;
use crate::piece::PieceKind;
use crate::position::Position;

/// Return true if making `m` from `pos` leaves the side to move in check.
pub fn is_legal(pos: &Position, m: &ChessMove) -> bool {
    // `apply_move_into` overwrites all board/state fields, so start from a
    // cheap stack copy instead of constructing the default position (FEN parse).
    let mut new_pos = *pos;
    pos.apply_move_into(m, &mut new_pos);

    // Preserve prior contract: if the mover's king is missing after make-move,
    // this candidate is invalid and must be rejected.
    if new_pos
        .pieces
        .get(Piece::from_parts(pos.side_to_move, Some(PieceKind::King)))
        .first_square()
        .is_none()
    {
        return false;
    }

    !is_in_check(&new_pos, pos.side_to_move)
}

/// Fast legality check when position after move is already computed
/// This avoids redundant position copies in tight loops
#[inline]
pub fn is_legal_fast(original_pos: &Position, pos_after_move: &Position) -> bool {
    // Check if the mover's king is missing after make-move
    if pos_after_move
        .pieces
        .get(Piece::from_parts(
            original_pos.side_to_move,
            Some(PieceKind::King),
        ))
        .first_square()
        .is_none()
    {
        return false;
    }

    !is_in_check(pos_after_move, original_pos.side_to_move)
}

#[inline(always)]
pub fn is_in_check(pos: &Position, color: Color) -> bool {
    let king_sq = match pos
        .pieces
        .get(Piece::from_parts(color, Some(PieceKind::King)))
        .first_square()
    {
        Some(sq) => sq,
        None => return false,
    };

    is_square_attacked_by(pos, king_sq, color.opposite())
}

#[inline(always)]
fn is_square_attacked_by(pos: &Position, sq: Square, attacker_color: Color) -> bool {
    // Pawn attacks
    if pawn_attacks_to(sq, attacker_color)
        .and(
            pos.pieces
                .get(Piece::from_parts(attacker_color, Some(PieceKind::Pawn))),
        )
        .is_nonempty()
    {
        return true;
    }

    // Knight attacks
    if knight_attacks(sq)
        .and(
            pos.pieces
                .get(Piece::from_parts(attacker_color, Some(PieceKind::Knight))),
        )
        .is_nonempty()
    {
        return true;
    }

    // Bishop/Queen attacks
    if bishop_attacks_from(sq, pos.occupancy[OccupancyKind::Both])
        .and(
            pos.pieces
                .get(Piece::from_parts(attacker_color, Some(PieceKind::Bishop)))
                | pos
                    .pieces
                    .get(Piece::from_parts(attacker_color, Some(PieceKind::Queen))),
        )
        .is_nonempty()
    {
        return true;
    }

    // Rook/Queen attacks
    if rook_attacks_from(sq, pos.occupancy[OccupancyKind::Both])
        .and(
            pos.pieces
                .get(Piece::from_parts(attacker_color, Some(PieceKind::Rook)))
                | pos
                    .pieces
                    .get(Piece::from_parts(attacker_color, Some(PieceKind::Queen))),
        )
        .is_nonempty()
    {
        return true;
    }

    // King attacks
    king_attacks(sq)
        .and(
            pos.pieces
                .get(Piece::from_parts(attacker_color, Some(PieceKind::King))),
        )
        .is_nonempty()
}

use crate::BitBoardMask;
use crate::Square;
use crate::bitboard::bishop_attacks_from;
use crate::bitboard::king_attacks;
use crate::bitboard::knight_attacks;
use crate::bitboard::pawn_attacks_to;
use crate::bitboard::rook_attacks_from;
use crate::mov::ChessMove;
use crate::mov::MoveType;
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

pub fn is_in_check(pos: &Position, color: Color) -> bool {
    let king_piece = Piece::from_parts(color, Some(PieceKind::King));
    let king_sq = match pos.pieces.get(king_piece).first_square() {
        Some(sq) => sq,
        None => return false,
    };

    is_square_attacked_by(pos, king_sq, color.opposite())
}

fn is_square_attacked_by(pos: &Position, sq: Square, attacker_color: Color) -> bool {
    let attacker_pawns = pos
        .pieces
        .get(Piece::from_parts(attacker_color, Some(PieceKind::Pawn)));

    // Pawn attacks
    if pawn_attacks_to(sq, attacker_color)
        .and(attacker_pawns)
        .is_nonempty()
    {
        return true;
    }

    // Knight attacks
    let attacker_knights = pos
        .pieces
        .get(Piece::from_parts(attacker_color, Some(PieceKind::Knight)));
    if knight_attacks(sq).and(attacker_knights).is_nonempty() {
        return true;
    }

    // Bishop/Queen attacks
    let occ = pos.occupancy[OccupancyKind::Both];
    let attacker_bishops = pos
        .pieces
        .get(Piece::from_parts(attacker_color, Some(PieceKind::Bishop)));
    let attacker_queens = pos
        .pieces
        .get(Piece::from_parts(attacker_color, Some(PieceKind::Queen)));
    if bishop_attacks_from(sq, occ)
        .and(attacker_bishops | attacker_queens)
        .is_nonempty()
    {
        return true;
    }

    // Rook/Queen attacks
    let attacker_rooks = pos
        .pieces
        .get(Piece::from_parts(attacker_color, Some(PieceKind::Rook)));
    if rook_attacks_from(sq, occ)
        .and(attacker_rooks | attacker_queens)
        .is_nonempty()
    {
        return true;
    }

    // King attacks
    let attacker_king = pos
        .pieces
        .get(Piece::from_parts(attacker_color, Some(PieceKind::King)));
    king_attacks(sq).and(attacker_king).is_nonempty()
}

/// Optimized legality check using bitboard operations without mutating
/// position. This is significantly faster than make/unmake for legality
/// testing.
///
/// Returns true if the move is legal (doesn't leave own king in check).
pub fn is_move_legal_without_making(pos: &Position, mv: &ChessMove) -> bool {
    let us = pos.side_to_move;
    let them = us.opposite();

    // Find our king
    let king_piece = Piece::from_parts(us, Some(PieceKind::King));
    let king_sq = match pos.pieces.get(king_piece).first_square() {
        Some(sq) => sq,
        None => return false, // No king = illegal position
    };

    match mv.move_type {
        MoveType::CastleKingside | MoveType::CastleQueenside => {
            // Castling legality requires checking multiple squares
            // The king cannot castle through check or into check
            check_castling_legality(pos, mv, us, them, king_sq)
        }
        _ => {
            // For all other moves, check if the king would be in check after the move
            let is_king_move = mv.from == king_sq;

            if is_king_move {
                // King moves: check if destination is attacked
                !is_square_attacked_after_king_move(pos, mv.to, us, them)
            } else {
                // Non-king moves: simulate occupancy change and check if king is attacked
                check_legality_with_simulated_occupancy(pos, mv, king_sq, us, them)
            }
        }
    }
}

/// Check if a square is attacked after the king moves there.
fn is_square_attacked_after_king_move(
    pos: &Position,
    king_to: Square,
    us: Color,
    them: Color,
) -> bool {
    // Simulate occupancy with king on the new square
    let from_mask = BitBoardMask::from_square(
        pos.pieces
            .get(Piece::from_parts(us, Some(PieceKind::King)))
            .first_square()
            .unwrap(),
    );
    let to_mask = BitBoardMask::from_square(king_to);
    let new_occ = (pos.occupancy[OccupancyKind::Both] & !from_mask) | to_mask;

    is_square_attacked_with_occupancy(pos, king_to, them, new_occ)
}

/// Check legality for non-king moves by simulating the occupancy change.
fn check_legality_with_simulated_occupancy(
    pos: &Position,
    mv: &ChessMove,
    king_sq: Square,
    us: Color,
    them: Color,
) -> bool {
    let from_mask = BitBoardMask::from_square(mv.from);
    let to_mask = BitBoardMask::from_square(mv.to);

    // Handle en passant capture specially
    let captured_sq_mask = match mv.move_type {
        MoveType::EnPassant => {
            // The captured pawn is not on the 'to' square
            let ep_capture_sq = match us {
                Color::White => mv.to.backward(1).unwrap(),
                Color::Black => mv.to.forward(1).unwrap(),
            };
            BitBoardMask::from_square(ep_capture_sq)
        }
        _ => to_mask, // Normal capture removes piece at 'to' square
    };

    // Simulate the move: remove from 'from', add to 'to', remove captured piece
    let new_occ = (pos.occupancy[OccupancyKind::Both] & !from_mask & !captured_sq_mask) | to_mask;

    // Check if our king is attacked with the new occupancy
    !is_square_attacked_with_occupancy(pos, king_sq, them, new_occ)
}

/// Check castling legality (king not in check, not castling through check, not
/// castling into check).
fn check_castling_legality(
    pos: &Position,
    mv: &ChessMove,
    us: Color,
    them: Color,
    king_sq: Square,
) -> bool {
    // King must not be in check before castling
    if is_square_attacked_by(pos, king_sq, them) {
        return false;
    }

    // Check the squares the king passes through and lands on
    let (middle_sq, target_sq) = match (mv.move_type, us) {
        (MoveType::CastleKingside, Color::White) => (Square::F1, Square::G1),
        (MoveType::CastleKingside, Color::Black) => (Square::F8, Square::G8),
        (MoveType::CastleQueenside, Color::White) => (Square::D1, Square::C1),
        (MoveType::CastleQueenside, Color::Black) => (Square::D8, Square::C8),
        _ => return false,
    };

    // King cannot castle through check or into check
    !is_square_attacked_by(pos, middle_sq, them) && !is_square_attacked_by(pos, target_sq, them)
}

/// Check if a square is attacked by the opponent with a specific occupancy
/// bitboard. This allows us to test attacks with simulated board changes
/// without mutating the position.
fn is_square_attacked_with_occupancy(
    pos: &Position,
    sq: Square,
    attacker_color: Color,
    occ: BitBoardMask,
) -> bool {
    let attacker_pawns = pos
        .pieces
        .get(Piece::from_parts(attacker_color, Some(PieceKind::Pawn)));
    let attacker_knights = pos
        .pieces
        .get(Piece::from_parts(attacker_color, Some(PieceKind::Knight)));
    let attacker_bishops = pos
        .pieces
        .get(Piece::from_parts(attacker_color, Some(PieceKind::Bishop)));
    let attacker_rooks = pos
        .pieces
        .get(Piece::from_parts(attacker_color, Some(PieceKind::Rook)));
    let attacker_queens = pos
        .pieces
        .get(Piece::from_parts(attacker_color, Some(PieceKind::Queen)));
    let attacker_king = pos
        .pieces
        .get(Piece::from_parts(attacker_color, Some(PieceKind::King)));

    // Pawn attacks (independent of occupancy)
    if pawn_attacks_to(sq, attacker_color)
        .and(attacker_pawns)
        .is_nonempty()
    {
        return true;
    }

    // Knight attacks (independent of occupancy)
    if knight_attacks(sq).and(attacker_knights).is_nonempty() {
        return true;
    }

    // Bishop/Queen attacks (depend on occupancy)
    if bishop_attacks_from(sq, occ)
        .and(attacker_bishops | attacker_queens)
        .is_nonempty()
    {
        return true;
    }

    // Rook/Queen attacks (depend on occupancy)
    if rook_attacks_from(sq, occ)
        .and(attacker_rooks | attacker_queens)
        .is_nonempty()
    {
        return true;
    }

    // King attacks (independent of occupancy)
    king_attacks(sq).and(attacker_king).is_nonempty()
}

use crate::BitBoardMask;
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
    let mut new_pos = Position::default();
    pos.apply_move_into(m, &mut new_pos);

    // Try to find the king square for the original side to move
    let king_sq_opt = new_pos
        .pieces
        .get(Piece::from_parts(pos.side_to_move, Some(PieceKind::King)))
        .squares()
        .next();

    if king_sq_opt.is_none() {
        return false;
    }

    let king_sq = king_sq_opt.unwrap();
    let attackers = get_attackers(&new_pos, king_sq, pos.side_to_move.opposite());

    attackers.is_empty()
}

fn get_attackers(pos: &Position, sq: Square, attacker_color: Color) -> BitBoardMask {
    let mut attackers = BitBoardMask::empty();

    // Pawn attacks
    attackers |= pawn_attacks_to(sq, attacker_color).and(
        pos.pieces
            .get(Piece::from_parts(attacker_color, Some(PieceKind::Pawn))),
    );

    // Knight attacks
    attackers |= knight_attacks(sq).and(
        pos.pieces
            .get(Piece::from_parts(attacker_color, Some(PieceKind::Knight))),
    );

    // Bishop/Queen attacks
    attackers |= bishop_attacks_from(sq, pos.occupancy[OccupancyKind::Both]).and(
        pos.pieces
            .get(Piece::from_parts(attacker_color, Some(PieceKind::Bishop)))
            | pos
                .pieces
                .get(Piece::from_parts(attacker_color, Some(PieceKind::Queen))),
    );

    // Rook/Queen attacks
    attackers |= rook_attacks_from(sq, pos.occupancy[OccupancyKind::Both]).and(
        pos.pieces
            .get(Piece::from_parts(attacker_color, Some(PieceKind::Rook)))
            | pos
                .pieces
                .get(Piece::from_parts(attacker_color, Some(PieceKind::Queen))),
    );

    // King attacks
    attackers |= king_attacks(sq).and(
        pos.pieces
            .get(Piece::from_parts(attacker_color, Some(PieceKind::King))),
    );
    attackers
}

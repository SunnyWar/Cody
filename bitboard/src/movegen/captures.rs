use crate::MoveList;
use crate::bitboard::PAWN_ATTACKS;
use crate::bitboard::bishop_attacks_from;
use crate::bitboard::king_attacks;
use crate::bitboard::knight_attacks;
use crate::bitboard::pawn_attacks_to;
use crate::bitboard::rook_attacks_from;
use crate::mov::ChessMove;
use crate::mov::MoveType;
use crate::piece::Color;
use crate::piece::Piece;
use crate::piece::PieceKind;
use crate::position::Position;

/// Fast zero-allocation pseudo capture generation
pub fn generate_pseudo_captures_fast(pos: &Position) -> MoveList {
    let mut moves = MoveList::new();
    let us = pos.side_to_move;
    let their_occ = pos.their_pieces(us);
    let occ = pos.all_pieces();
    let promo_from_rank = if us == Color::White { 6 } else { 1 };

    // Pawn captures (including promotions)
    let pawn_bb = pos.pieces.get(Piece::from_parts(us, Some(PieceKind::Pawn)));
    for from in pawn_bb.squares() {
        let attacks = PAWN_ATTACKS[us.index()][from.index()] & their_occ;
        let is_promo = from.rank() == promo_from_rank;
        for to in attacks.squares() {
            if is_promo {
                for &promo in &[
                    PieceKind::Queen,
                    PieceKind::Rook,
                    PieceKind::Bishop,
                    PieceKind::Knight,
                ] {
                    moves.push(ChessMove::new(from, to, MoveType::Promotion(promo)));
                }
            } else {
                moves.push(ChessMove::new(from, to, MoveType::Capture));
            }
        }
    }

    // Knight captures
    let knight_bb = pos
        .pieces
        .get(Piece::from_parts(us, Some(PieceKind::Knight)));
    for from in knight_bb.squares() {
        let attacks = knight_attacks(from) & their_occ;
        for to in attacks.squares() {
            moves.push(ChessMove::new(from, to, MoveType::Capture));
        }
    }

    // Bishop/queen captures
    let bishop_like_bb = pos
        .pieces
        .get(Piece::from_parts(us, Some(PieceKind::Bishop)))
        | pos
            .pieces
            .get(Piece::from_parts(us, Some(PieceKind::Queen)));
    for from in bishop_like_bb.squares() {
        let attacks = bishop_attacks_from(from, occ) & their_occ;
        for to in attacks.squares() {
            moves.push(ChessMove::new(from, to, MoveType::Capture));
        }
    }

    // Rook/queen captures
    let rook_like_bb = pos.pieces.get(Piece::from_parts(us, Some(PieceKind::Rook)))
        | pos
            .pieces
            .get(Piece::from_parts(us, Some(PieceKind::Queen)));
    for from in rook_like_bb.squares() {
        let attacks = rook_attacks_from(from, occ) & their_occ;
        for to in attacks.squares() {
            moves.push(ChessMove::new(from, to, MoveType::Capture));
        }
    }

    // King captures
    let king_bb = pos.pieces.get(Piece::from_parts(us, Some(PieceKind::King)));
    if let Some(from) = king_bb.squares().next() {
        let attacks = king_attacks(from) & their_occ;
        for to in attacks.squares() {
            moves.push(ChessMove::new(from, to, MoveType::Capture));
        }
    }

    // En-passant captures
    if let Some(ep_sq) = pos.ep_square {
        let ep_attackers = pawn_attacks_to(ep_sq, us) & pawn_bb;
        for from in ep_attackers.squares() {
            moves.push(ChessMove::new(from, ep_sq, MoveType::EnPassant));
        }
    }

    moves
}

/// Backward-compatible Vec-based capture generation
/// Generate pseudo capture-like moves (captures, promotions, en-passant).
pub fn generate_pseudo_captures(pos: &Position) -> Vec<ChessMove> {
    generate_pseudo_captures_fast(pos).to_vec()
}

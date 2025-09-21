use crate::{
    Square,
    bitboard::{
        bishop_attacks_from, king_attacks, knight_attacks, pawn_attacks_to, rook_attacks_from,
    },
    mov::ChessMove,
    mov::MoveType,
    piece::Color,
    piece::{Piece, PieceKind},
    position::Position,
};

/// Generate pseudo capture-like moves (captures, promotions, en-passant).
pub fn generate_pseudo_captures(pos: &Position) -> Vec<ChessMove> {
    use crate::bitboard::{
        bishop_attacks_from, king_attacks, knight_attacks, pawn_attacks_to, rook_attacks_from,
    };
    use crate::mov::MoveType;
    use crate::piece::PieceKind;

    let mut moves = Vec::new();
    let us = pos.side_to_move;
    let their_occ = pos.their_pieces(us);
    let occ = pos.all_pieces();

    // Pawn captures (including promotions)
    let pawn_bb = pos.pieces.get(Piece::from_parts(us, Some(PieceKind::Pawn)));
    for to in Square::all_array() {
        // only consider pawn attacks that actually capture an opponent piece
        let attackers = pawn_attacks_to(to, us) & pawn_bb & their_occ;
        for from in attackers.squares() {
            let is_promo =
                (us == Color::White && to.rank() == 7) || (us == Color::Black && to.rank() == 0);
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

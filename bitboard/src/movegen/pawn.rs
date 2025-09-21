use crate::{
    Square,
    constants::{
        DOUBLE_NORTH, DOUBLE_SOUTH, NORTH, NORTH_EAST, NORTH_WEST, SOUTH, SOUTH_EAST, SOUTH_WEST,
    },
    mov::{ChessMove, MoveType},
    piece::{Color, Piece, PieceKind},
    position::MoveGenContext,
    position::Position,
    tables::{
        file_masks::{FILE_A, FILE_H},
        rank_masks::{RANK_4, RANK_5},
    },
};

fn is_promotion_rank(square: Square, color: Color) -> bool {
    match color {
        Color::White => square.rank() == 7,
        Color::Black => square.rank() == 0,
    }
}

pub fn generate_pseudo_pawn_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<ChessMove>,
) {
    let pawns = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Pawn)));
    if pawns.is_empty() {
        return;
    }

    let empty = !context.occupancy;
    let their_pieces = pos.their_pieces(context.us);

    let (single_push_dir, double_push_dir, left_cap_dir, right_cap_dir, double_rank_mask) =
        match context.us {
            Color::White => (NORTH, DOUBLE_NORTH, NORTH_WEST, NORTH_EAST, RANK_4),
            Color::Black => (SOUTH, DOUBLE_SOUTH, SOUTH_EAST, SOUTH_WEST, RANK_5),
        };

    // Single push
    let single_push = (pawns << single_push_dir) & empty;
    for to in single_push.squares() {
        if let Some(from) = to.advance(-single_push_dir) {
            if is_promotion_rank(to, context.us) {
                for &promo in &[
                    PieceKind::Queen,
                    PieceKind::Rook,
                    PieceKind::Bishop,
                    PieceKind::Knight,
                ] {
                    moves.push(ChessMove::new(from, to, MoveType::Promotion(promo)));
                }
            } else {
                moves.push(ChessMove::new(from, to, MoveType::Quiet));
            }
        }
    }

    // Double push (never a promotion)
    let double_push = ((single_push << single_push_dir) & empty) & double_rank_mask;
    for to in double_push.squares() {
        if let Some(from) = to.advance(-double_push_dir) {
            moves.push(ChessMove::new(from, to, MoveType::Quiet));
        }
    }

    // Left capture
    let left_caps = (pawns << left_cap_dir) & their_pieces & !FILE_H;
    for to in left_caps.squares() {
        if let Some(from) = to.advance(-left_cap_dir) {
            if is_promotion_rank(to, context.us) {
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

    // Right capture
    let right_caps = (pawns << right_cap_dir) & their_pieces & !FILE_A;
    for to in right_caps.squares() {
        if let Some(from) = to.advance(-right_cap_dir) {
            if is_promotion_rank(to, context.us) {
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

    // En passant
    if let Some(ep_square) = pos.ep_square {
        // Left EP capture
        let left_ep = (pawns << left_cap_dir) & ep_square.bitboard();
        for to in left_ep.squares() {
            if let Some(from) = to.advance(-left_cap_dir) {
                moves.push(ChessMove::new(from, to, MoveType::EnPassant));
            }
        }

        // Right EP capture
        let right_ep = (pawns << right_cap_dir) & ep_square.bitboard();
        for to in right_ep.squares() {
            if let Some(from) = to.advance(-right_cap_dir) {
                moves.push(ChessMove::new(from, to, MoveType::EnPassant));
            }
        }
    }
}

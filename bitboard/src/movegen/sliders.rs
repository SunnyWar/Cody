use crate::{
    bitboard::{bishop_attacks_from, rook_attacks_from},
    mov::ChessMove,
    piece::{Piece, PieceKind},
    position::{MoveGenContext, Position},
};

pub fn generate_pseudo_bishop_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<ChessMove>,
) {
    let bishops = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Bishop)));

    for from in bishops.squares() {
        let attacks = bishop_attacks_from(from, context.occupancy);
        let valid_moves = attacks.and(context.not_ours);
        crate::movegen::api::push_moves_from_valid_targets(pos, context, from, valid_moves, moves);
    }
}

pub fn generate_pseudo_rook_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<ChessMove>,
) {
    let rooks = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Rook)));

    for from in rooks.squares() {
        let valid_moves = rook_attacks_from(from, context.occupancy).and(context.not_ours);
        crate::movegen::api::push_moves_from_valid_targets(pos, context, from, valid_moves, moves);
    }
}

pub fn generate_pseudo_queen_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<ChessMove>,
) {
    let queens = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Queen)));

    for from in queens.squares() {
        let valid_moves = (rook_attacks_from(from, context.occupancy)
            | bishop_attacks_from(from, context.occupancy))
        .and(context.not_ours);

        crate::movegen::api::push_moves_from_valid_targets(pos, context, from, valid_moves, moves);
    }
}

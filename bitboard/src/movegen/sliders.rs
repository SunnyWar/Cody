use crate::MoveList;
use crate::bitboard::bishop_attacks_from;
use crate::bitboard::rook_attacks_from;
use crate::mov::ChessMove;
use crate::piece::Piece;
use crate::piece::PieceKind;
use crate::position::MoveGenContext;
use crate::position::Position;

pub fn generate_pseudo_bishop_moves_fast(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut MoveList,
) {
    let bishops = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Bishop)));

    if bishops.is_empty() {
        return;
    }

    for from in bishops.squares() {
        let attacks = bishop_attacks_from(from, context.occupancy);
        let valid_moves = attacks.and(context.not_ours);
        crate::movegen::api::push_moves_from_valid_targets_fast(
            pos,
            context,
            from,
            valid_moves,
            moves,
        );
    }
}

pub fn generate_pseudo_rook_moves_fast(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut MoveList,
) {
    let rooks = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Rook)));

    if rooks.is_empty() {
        return;
    }

    for from in rooks.squares() {
        let valid_moves = rook_attacks_from(from, context.occupancy).and(context.not_ours);
        crate::movegen::api::push_moves_from_valid_targets_fast(
            pos,
            context,
            from,
            valid_moves,
            moves,
        );
    }
}

pub fn generate_pseudo_queen_moves_fast(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut MoveList,
) {
    let queens = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Queen)));

    if queens.is_empty() {
        return;
    }

    for from in queens.squares() {
        let valid_moves = (rook_attacks_from(from, context.occupancy)
            | bishop_attacks_from(from, context.occupancy))
        .and(context.not_ours);

        crate::movegen::api::push_moves_from_valid_targets_fast(
            pos,
            context,
            from,
            valid_moves,
            moves,
        );
    }
}

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

/// Optimized queen move generation - now benefits from fast `bishop_attacks_from`
pub fn generate_pseudo_queen_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<ChessMove>,
) {
    let queens = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Queen)));

    for from in queens.squares() {
        // Both functions now use fast table lookups after optimization
        let valid_moves = (rook_attacks_from(from, context.occupancy)
            | bishop_attacks_from(from, context.occupancy))
        .and(context.not_ours);

        crate::movegen::api::push_moves_from_valid_targets(pos, context, from, valid_moves, moves);
    }
}

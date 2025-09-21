use crate::{
    bitboard::knight_attacks,
    mov::ChessMove,
    piece::{Piece, PieceKind},
    position::{MoveGenContext, Position},
};

pub fn generate_pseudo_knight_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<ChessMove>,
) {
    // Get a bitboard of all knights for the current side.
    let knights = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Knight)));

    // Iterate over each square containing one of our knights.
    for from in knights.squares() {
        // Calculate all squares this knight can move to, filtered by squares
        // not occupied by our own pieces.
        let valid_moves = knight_attacks(from).and(context.not_ours);
        crate::movegen::api::push_moves_from_valid_targets(pos, context, from, valid_moves, moves);
    }
}

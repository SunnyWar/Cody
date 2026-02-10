use crate::bitboard::knight_attacks;
use crate::mov::ChessMove;
use crate::piece::Piece;
use crate::piece::PieceKind;
use crate::position::MoveGenContext;
use crate::position::Position;

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
        let attacks = knight_attacks(from);
        let valid_moves = attacks.and(context.not_ours);
        crate::movegen::api::push_moves_from_valid_targets(pos, context, from, valid_moves, moves);
    }
}

use crate::{
    Square,
    bitboard::king_attacks,
    mov::{ChessMove, MoveType},
    piece::{Color, Piece, PieceKind},
    position::{MoveGenContext, Position},
};

pub fn generate_pseudo_king_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<ChessMove>,
) {
    let king_bb = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::King)));

    if let Some(from) = king_bb.squares().next() {
        // Standard king moves
        let valid_moves = king_attacks(from).and(context.not_ours);
        crate::movegen::api::push_moves_from_valid_targets(pos, context, from, valid_moves, moves);

        // Castling moves
        if pos.can_castle_kingside(context.us) {
            let to = match context.us {
                Color::White => Square::G1,
                Color::Black => Square::G8,
            };
            moves.push(ChessMove::new(from, to, MoveType::CastleKingside));
        }

        if pos.can_castle_queenside(context.us) {
            let to = match context.us {
                Color::White => Square::C1,
                Color::Black => Square::C8,
            };
            moves.push(ChessMove::new(from, to, MoveType::CastleQueenside));
        }
    }
}

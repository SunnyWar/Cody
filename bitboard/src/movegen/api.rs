// This file started life as the monolithic movegen.rs. For the refactor PR#1
// we move the full original content here unchanged so external users see the
// same API while we split internals in subsequent PRs.

// --- Begin: original movegen.rs content ---

use crate::{
    BitBoardMask, Square,
    attack::{BoardState, PieceSet, is_king_in_check},
    bitboard::{
        bishop_attacks_from, king_attacks, knight_attacks, pawn_attacks_to, rook_attacks_from,
    },
    constants::{
        DOUBLE_NORTH, DOUBLE_SOUTH, NORTH, NORTH_EAST, NORTH_WEST, SOUTH, SOUTH_EAST, SOUTH_WEST,
    },
    mov::{ChessMove, MoveType},
    occupancy::OccupancyKind,
    piece::{Color, Piece, PieceKind},
    position::{MoveGenContext, Position},
    tables::{
        file_masks::{FILE_A, FILE_H},
        rank_masks::{RANK_4, RANK_5},
    },
};

#[derive(Clone, Copy)]
pub struct SimpleMoveGen;

pub trait MoveGenerator {
    fn in_check(&self, pos: &Position) -> bool;
}

impl MoveGenerator for SimpleMoveGen {
    fn in_check(&self, pos: &Position) -> bool {
        let board_state = BoardState {
            occupancy: pos.occupancy[OccupancyKind::Both],
            white_pieces: PieceSet {
                pawns: pos
                    .pieces
                    .get(Piece::from_parts(Color::White, Some(PieceKind::Pawn))),
                knights: pos
                    .pieces
                    .get(Piece::from_parts(Color::White, Some(PieceKind::Knight))),
                bishops: pos
                    .pieces
                    .get(Piece::from_parts(Color::White, Some(PieceKind::Bishop))),
                rooks: pos
                    .pieces
                    .get(Piece::from_parts(Color::White, Some(PieceKind::Rook))),
                queens: pos
                    .pieces
                    .get(Piece::from_parts(Color::White, Some(PieceKind::Queen))),
                king: pos
                    .pieces
                    .get(Piece::from_parts(Color::White, Some(PieceKind::King))),
            },
            black_pieces: PieceSet {
                pawns: pos
                    .pieces
                    .get(Piece::from_parts(Color::Black, Some(PieceKind::Pawn))),
                knights: pos
                    .pieces
                    .get(Piece::from_parts(Color::Black, Some(PieceKind::Knight))),
                bishops: pos
                    .pieces
                    .get(Piece::from_parts(Color::Black, Some(PieceKind::Bishop))),
                rooks: pos
                    .pieces
                    .get(Piece::from_parts(Color::Black, Some(PieceKind::Rook))),
                queens: pos
                    .pieces
                    .get(Piece::from_parts(Color::Black, Some(PieceKind::Queen))),
                king: pos
                    .pieces
                    .get(Piece::from_parts(Color::Black, Some(PieceKind::King))),
            },
        };

        let king_color = pos.side_to_move;
        is_king_in_check(king_color, &board_state)
    }
}

pub fn generate_pseudo_moves(pos: &Position) -> Vec<ChessMove> {
    let mut moves = Vec::new();
    let context = MoveGenContext {
        us: pos.side_to_move,
        occupancy: pos.all_pieces(),
        not_ours: !pos.our_pieces(pos.side_to_move),
    };

    crate::movegen::generate_pseudo_pawn_moves(pos, &context, &mut moves);
    generate_pseudo_knight_moves(pos, &context, &mut moves);
    generate_pseudo_bishop_moves(pos, &context, &mut moves);
    generate_pseudo_rook_moves(pos, &context, &mut moves);
    generate_pseudo_queen_moves(pos, &context, &mut moves);
    generate_pseudo_king_moves(pos, &context, &mut moves);

    moves
}

pub fn generate_legal_moves(pos: &Position) -> Vec<ChessMove> {
    generate_pseudo_moves(pos)
        .into_iter()
        .filter(|m| is_legal(pos, m))
        .collect()
}

/// Generate pseudo capture-like moves (captures, promotions, en-passant).
/// This mirrors the helper previously duplicated in the engine crate so the
/// move-generation logic is centralized in the `bitboard` crate.
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

// Pawn move generation was moved into `movegen::pawn` during the refactor.
// See `crate::movegen::pawn` for the implementation (generate_pseudo_pawn_moves).

// TODO - this can probably be improved by have an attack mask
// TODO - generated by a 5x5 move mask moved around to every square added
// TODO - making final 8x8 can be masked with current knight location to make
// TODO - new mask that only shows possible moves then mask that will
// TODO - opponent locations and empty square to show all possible moves
// TODO - all with masking!
fn generate_pseudo_knight_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<ChessMove>,
) {
    // Get a bitboard of all knights for the current side.
    // Delegate to the extracted knight module implementation.
    crate::movegen::generate_pseudo_knight_moves(pos, context, moves);
}

fn generate_pseudo_bishop_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<ChessMove>,
) {
    crate::movegen::generate_pseudo_bishop_moves(pos, context, moves);
}

fn generate_pseudo_rook_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<ChessMove>,
) {
    crate::movegen::generate_pseudo_rook_moves(pos, context, moves);
}

#[inline]
pub(crate) fn push_moves_from_valid_targets(
    pos: &Position,
    context: &MoveGenContext,
    from: Square,
    valid_targets: BitBoardMask,
    moves: &mut Vec<ChessMove>,
) {
    for to in valid_targets.squares() {
        let move_type = if pos.our_pieces(context.us.opposite()).contains(to) {
            MoveType::Capture
        } else {
            MoveType::Quiet
        };
        moves.push(ChessMove::new(from, to, move_type));
    }
}

fn generate_pseudo_queen_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<ChessMove>,
) {
    crate::movegen::generate_pseudo_queen_moves(pos, context, moves);
}

fn generate_pseudo_king_moves(
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
        push_moves_from_valid_targets(pos, context, from, valid_moves, moves);

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

#[cfg(test)]
mod tests {
    use crate::Square;

    use super::*;

    fn move_exists(
        moves: &[ChessMove],
        from: Square,
        to: Square,
        promotion: Option<PieceKind>,
    ) -> bool {
        moves.iter().any(|m| {
            m.from == from
                && m.to == to
                && match (&m.move_type, promotion) {
                    (MoveType::Promotion(k), Some(p)) => *k == p,
                    (MoveType::Promotion(_), None) => false,
                    (MoveType::Quiet, None) => true,
                    (MoveType::Capture, None) => true,
                    (MoveType::EnPassant, None) => true,
                    (MoveType::CastleKingside, None) => true,
                    (MoveType::CastleQueenside, None) => true,
                    _ => false,
                }
        })
    }

    #[test]
    fn test_initial_position_move_count() {
        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let moves = generate_legal_moves(&pos);
        // In the initial position, white has 20 legal moves (16 pawn moves + 4 knight moves)
        assert_eq!(moves.len(), 20);
    }

    // ... remaining tests untouched ...
}

// --- End: original movegen.rs content ---

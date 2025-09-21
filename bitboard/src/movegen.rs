// bitboard/src/movegen.rs

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

    generate_pseudo_pawn_moves(pos, &context, &mut moves);
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
    use crate::bitboard::{bishop_attacks_from, king_attacks, knight_attacks, pawn_attacks_to, rook_attacks_from};
    use crate::mov::MoveType;
    use crate::piece::PieceKind;

    let mut moves = Vec::new();
    let us = pos.side_to_move;
    let their_occ = pos.their_pieces(us);
    let occ = pos.all_pieces();

    // Pawn captures (including promotions)
    let pawn_bb = pos.pieces.get(Piece::from_parts(us, Some(PieceKind::Pawn)));
    for to in Square::all_array() {
        let attackers = pawn_attacks_to(to, us) & pawn_bb;
        for from in attackers.squares() {
            let is_promo = (us == Color::White && to.rank() == 7)
                || (us == Color::Black && to.rank() == 0);
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
    let knight_bb = pos.pieces.get(Piece::from_parts(us, Some(PieceKind::Knight)));
    for from in knight_bb.squares() {
        let attacks = knight_attacks(from) & their_occ;
        for to in attacks.squares() {
            moves.push(ChessMove::new(from, to, MoveType::Capture));
        }
    }

    // Bishop/queen captures
    let bishop_like_bb = pos.pieces.get(Piece::from_parts(us, Some(PieceKind::Bishop)))
        | pos.pieces.get(Piece::from_parts(us, Some(PieceKind::Queen)));
    for from in bishop_like_bb.squares() {
        let attacks = bishop_attacks_from(from, occ) & their_occ;
        for to in attacks.squares() {
            moves.push(ChessMove::new(from, to, MoveType::Capture));
        }
    }

    // Rook/queen captures
    let rook_like_bb = pos.pieces.get(Piece::from_parts(us, Some(PieceKind::Rook)))
        | pos.pieces.get(Piece::from_parts(us, Some(PieceKind::Queen)));
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

fn is_promotion_rank(square: Square, color: Color) -> bool {
    match color {
        Color::White => square.rank() == 7, // 8th rank
        Color::Black => square.rank() == 0, // 1st rank
    }
}

fn generate_pseudo_pawn_moves(
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
    let knights = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Knight)));

    // Iterate over each square containing one of our knights.
    for from in knights.squares() {
        // Calculate all squares this knight can move to, filtered by squares
        // not occupied by our own pieces.
        let valid_moves = knight_attacks(from).and(context.not_ours);
        push_moves_from_valid_targets(pos, context, from, valid_moves, moves);
    }
}

fn generate_pseudo_bishop_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<ChessMove>,
) {
    // Get a bitboard of all bishops for the current side.
    let bishops = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Bishop)));

    // Iterate over each square containing one of our bishops.
    for from in bishops.squares() {
        // Calculate all squares this bishop attacks, using the board's
        // total occupancy to identify blockers.
        let attacks = bishop_attacks_from(from, context.occupancy);

        // Filter these attacks to find valid moves (not occupied by our own pieces).
        let valid_moves = attacks.and(context.not_ours);
        push_moves_from_valid_targets(pos, context, from, valid_moves, moves);
    }
}

fn generate_pseudo_rook_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<ChessMove>,
) {
    // Get a bitboard of all rooks for the current side.
    let rooks = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Rook)));

    // Iterate over each square containing one of our rooks.
    for from in rooks.squares() {
        // Calculate all squares this rook attacks, using the board's total occupancy.
        let valid_moves = rook_attacks_from(from, context.occupancy).and(context.not_ours);
        push_moves_from_valid_targets(pos, context, from, valid_moves, moves);
    }
}

#[inline]
fn push_moves_from_valid_targets(
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
    // Get a bitboard of all queens for the current side.
    let queens = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Queen)));

    // Iterate over each square containing one of our queens.
    for from in queens.squares() {
        // Calculate all squares this queen can move to (rook-like and bishop-like moves).
        let valid_moves = (rook_attacks_from(from, context.occupancy)
            | bishop_attacks_from(from, context.occupancy))
        .and(context.not_ours);

        push_moves_from_valid_targets(pos, context, from, valid_moves, moves);
    }
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

    #[test]
    fn test_knight_moves() {
        let pos = Position::from_fen("4k3/8/8/8/8/8/3N4/4K3 w - - 0 1");
        let moves = generate_legal_moves(&pos);

        let from = Square::D2;
        let expected_targets = [
            Square::B1,
            Square::B3,
            Square::C4,
            Square::E4,
            Square::F3,
            Square::F1,
        ];

        for to in expected_targets.iter() {
            assert!(
                move_exists(&moves, from, *to, None),
                "Missing knight move to {:?}",
                to
            );
        }
    }

    #[test]
    fn test_generate_pseudo_pawn_moves() {
        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let context = MoveGenContext {
            us: Color::White,
            occupancy: pos.all_pieces(),
            not_ours: !pos.our_pieces(Color::White),
        };
        let mut moves = Vec::new();
        generate_pseudo_pawn_moves(&pos, &context, &mut moves);
        assert!(!moves.is_empty());
    }

    #[test]
    fn test_pawn_promotion() {
        let pos = Position::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1");
        let moves = generate_legal_moves(&pos);

        let from = Square::A7;
        let to = Square::A8;

        let promotions = [
            PieceKind::Queen,
            PieceKind::Rook,
            PieceKind::Bishop,
            PieceKind::Knight,
        ];

        for m in &moves {
            println!("{:?} -> {:?}, {:?}", m.from, m.to, m.move_type);
        }

        for kind in promotions.iter() {
            assert!(
                move_exists(&moves, from, to, Some(*kind)),
                "Missing promotion to {:?}",
                kind
            );
        }
    }

    #[test]
    fn test_king_moves_blocked() {
        let pos = Position::from_fen("8/8/8/8/8/8/8/4K3 w - - 0 1");
        let moves = generate_legal_moves(&pos);

        let from = Square::E1;
        let expected_targets = [
            Square::D1,
            Square::F1, // horizontal
            Square::D2,
            Square::E2,
            Square::F2, // vertical
        ];

        for to in expected_targets.iter() {
            assert!(
                move_exists(&moves, from, *to, None),
                "Missing king move to {:?}",
                to
            );
        }
    }

    #[test]
    fn test_rook_moves() {
        let pos = Position::from_fen("4k3/8/8/3R4/8/8/8/4K3 w - - 0 1");
        let moves = generate_legal_moves(&pos);

        let from = Square::D5;

        // Vertical moves along the d-file
        let expected_targets_vertical = [
            Square::D1,
            Square::D2,
            Square::D3,
            Square::D4,
            Square::D6,
            Square::D7,
            Square::D8,
        ];

        // Horizontal moves along rank 5
        let expected_targets_horizontal = [
            Square::A5,
            Square::B5,
            Square::C5,
            Square::E5,
            Square::F5,
            Square::G5,
            Square::H5,
        ];

        for to in expected_targets_vertical
            .iter()
            .chain(expected_targets_horizontal.iter())
        {
            assert!(
                move_exists(&moves, from, *to, None),
                "Missing rook move to {:?}",
                to
            );
        }
    }

    #[test]
    fn test_bishop_moves() {
        let pos = Position::from_fen("8/8/8/3B4/8/8/8/4K3 w - - 0 1");
        let moves = generate_legal_moves(&pos);

        let from = Square::D5;

        let expected_targets = [
            // Diagonals: up-left
            Square::C6,
            Square::B7,
            Square::A8,
            // Diagonals: up-right
            Square::E6,
            Square::F7,
            Square::G8,
            // Diagonals: down-left
            Square::C4,
            Square::B3,
            Square::A2,
            // Diagonals: down-right
            Square::E4,
            Square::F3,
            Square::G2,
            Square::H1,
        ];

        for to in expected_targets.iter() {
            assert!(
                move_exists(&moves, from, *to, None),
                "Missing bishop move to {:?}",
                to
            );
        }
    }

    #[test]
    fn test_queen_moves() {
        let pos = Position::from_fen("4k3/8/8/3Q4/8/8/8/4K3 w - - 0 1");
        let moves = generate_legal_moves(&pos);

        let from = Square::D5;

        let expected_targets = [
            // Vertical (file D)
            Square::D1,
            Square::D2,
            Square::D3,
            Square::D4,
            Square::D6,
            Square::D7,
            Square::D8,
            // Horizontal (rank 5)
            Square::A5,
            Square::B5,
            Square::C5,
            Square::E5,
            Square::F5,
            Square::G5,
            Square::H5,
            // Diagonals
            Square::C6,
            Square::B7,
            Square::A8, // up-left
            Square::E6,
            Square::F7,
            Square::G8, // up-right
            Square::C4,
            Square::B3,
            Square::A2, // down-left
            Square::E4,
            Square::F3,
            Square::G2,
            Square::H1, // down-right
        ];

        for to in expected_targets.iter() {
            assert!(
                move_exists(&moves, from, *to, None),
                "Missing queen move to {:?}",
                to
            );
        }
    }

    #[test]
    fn test_en_passant_capture() {
        // Black pawn just moved from d7 to d5, white can capture en passant with e5 pawn
        let pos = Position::from_fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");
        let moves = generate_legal_moves(&pos);

        let from = Square::E5;
        let to = Square::D6;

        assert!(
            move_exists(&moves, from, to, None),
            "Missing en passant capture from {:?} to {:?}",
            from,
            to
        );
    }

    #[test]
    fn test_white_kingside_castling() {
        let pos = Position::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        let moves = generate_legal_moves(&pos);

        let from = Square::E1;
        let to = Square::G1;

        assert!(
            move_exists(&moves, from, to, None),
            "Missing white kingside castling move from {:?} to {:?}",
            from,
            to
        );
    }

    #[test]
    fn test_white_queenside_castling() {
        let pos = Position::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        let moves = generate_legal_moves(&pos);

        let from = Square::E1;
        let to = Square::C1;

        assert!(
            move_exists(&moves, from, to, None),
            "Missing white queenside castling move from {:?} to {:?}",
            from,
            to
        );
    }

    #[test]
    fn test_black_kingside_castling() {
        let pos = Position::from_fen("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1");
        let moves = generate_legal_moves(&pos);

        let from = Square::E8;
        let to = Square::G8;

        assert!(
            move_exists(&moves, from, to, None),
            "Missing black kingside castling move from {:?} to {:?}",
            from,
            to
        );
    }

    #[test]
    fn test_black_queenside_castling() {
        let pos = Position::from_fen("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1");
        let moves = generate_legal_moves(&pos);

        let from = Square::E8;
        let to = Square::C8;

        assert!(
            move_exists(&moves, from, to, None),
            "Missing black queenside castling move from {:?} to {:?}",
            from,
            to
        );
    }

    #[test]
    fn test_knight_pinned_cannot_move() {
        // White knight on e2 is pinned by black rook on e8 through white king on e1
        let pos = Position::from_fen("4r3/8/8/8/8/8/4N3/4K3 w - - 0 1");
        let moves = generate_legal_moves(&pos);

        let pinned_knight = Square::E2;

        // Knight is pinned and should not be able to move
        assert!(
            !moves.iter().any(|m| m.from == pinned_knight),
            "Knight should not be able to move while pinned"
        );
    }

    #[test]
    fn test_pawn_pinned_cannot_advance() {
        // White pawn on e2 is pinned by black rook on b2 to white king on h2
        let pos = Position::from_fen("8/1k6/8/8/8/8/1r2P2K/8 w - - 0 1");
        let moves = generate_legal_moves(&pos);

        let pinned_pawn = Square::E2;

        // Pawn should not be able to move forward
        assert!(
            !moves.iter().any(|m| m.from == pinned_pawn),
            "Pinned pawn should not be able to advance"
        );
    }

    // TODO - fix
    #[test]
    fn test_rook_pinned_can_only_slide_along_pin() {
        // Rook on e2 pinned by queen on e8 through king on e1
        let pos = Position::from_fen("4q3/8/8/8/8/8/4R3/4K3 w - - 0 1");
        let moves = generate_legal_moves(&pos);

        let pinned_rook = Square::E2;
        let allowed_targets = [
            Square::E3,
            Square::E4,
            Square::E5,
            Square::E6,
            Square::E7,
            Square::E8, //capture
        ];

        for m in moves.iter().filter(|m| m.from == pinned_rook) {
            assert!(
                allowed_targets.contains(&m.to),
                "Pinned rook should only move along pin line, found move to {:?}",
                m.to
            );
        }
    }

    #[test]
    fn test_bishop_pinned_can_only_slide_along_diagonal() {
        // Bishop on d2 pinned by queen on g5 through king on c1
        let pos = Position::from_fen("8/8/8/6q1/8/8/3B4/2K5 w - - 0 1");
        let moves = generate_legal_moves(&pos);

        let pinned_bishop = Square::D2;
        let allowed_targets = [
            Square::E3,
            Square::F4,
            Square::G5,
            Square::C1, // assuming your engine allows sliding to the king square
        ];

        for m in moves.iter().filter(|m| m.from == pinned_bishop) {
            assert!(
                allowed_targets.contains(&m.to),
                "Pinned bishop should only move along diagonal, found move to {:?}",
                m.to
            );
        }
    }

    #[test]
    fn test_queen_pinned_can_only_slide_along_pin() {
        // Queen on d2 pinned by rook on d8 through king on d1
        let pos = Position::from_fen("3r4/8/8/8/8/8/3Q4/3K4 w - - 0 1");
        let moves = generate_legal_moves(&pos);

        let pinned_queen = Square::D2;
        let allowed_targets = [
            Square::D3,
            Square::D4,
            Square::D5,
            Square::D6,
            Square::D7,
            Square::D8, // capture
        ];

        for m in moves.iter().filter(|m| m.from == pinned_queen) {
            assert!(
                allowed_targets.contains(&m.to),
                "Pinned queen should only move along pin line, found move to {:?}",
                m.to
            );
        }
    }

    #[test]
    fn test_white_kingside_castling_blocked_by_attack() {
        // Black bishop on c4 attacks f1; white wants to castle kingside
        let pos = Position::from_fen("r3k2r/8/8/8/2b5/8/8/R3K2R w KQkq - 0 1");
        let moves = generate_legal_moves(&pos);

        let from = Square::E1;
        let to = Square::G1;

        // Castling should be disallowed because f1 is attacked
        assert!(
            !move_exists(&moves, from, to, None),
            "Castling should be disallowed due to attack on f1"
        );
    }

    #[test]
    fn test_white_queenside_castling_blocked_by_attack() {
        // Black bishop on g4 attacks d1; white wants to castle queenside
        let pos = Position::from_fen("r3k2r/8/8/8/6b1/8/8/R3K2R w KQkq - 0 1");
        let moves = generate_legal_moves(&pos);

        let from = Square::E1;
        let to = Square::C1;

        // Castling should be disallowed because d1 is attacked
        assert!(
            !move_exists(&moves, from, to, None),
            "Castling should be disallowed due to attack on d1"
        );
    }
}

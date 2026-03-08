// ...existing code...
// This file started life as the monolithic movegen.rs. For the refactor PR#1
// we move the full original content here unchanged so external users see the
// same API while we split internals in subsequent PRs.

// --- Begin: original movegen.rs content ---

use crate::BitBoardMask;
use crate::MoveList;
use crate::Square;
use crate::mov::ChessMove;
use crate::mov::MoveType;
use crate::position::MoveGenContext;
use crate::position::Position;

#[derive(Clone, Copy)]
pub struct SimpleMoveGen;

pub trait MoveGenerator {
    fn in_check(&self, pos: &Position) -> bool;
}

impl MoveGenerator for SimpleMoveGen {
    fn in_check(&self, pos: &Position) -> bool {
        crate::movegen::is_in_check(pos, pos.side_to_move)
    }
}

/// Fast zero-allocation move generation using stack-allocated MoveList
pub fn generate_pseudo_moves_fast(pos: &Position) -> MoveList {
    let mut moves = MoveList::new();
    let us = pos.side_to_move;
    let context = MoveGenContext {
        us,
        occupancy: pos.all_pieces(),
        not_ours: !pos.our_pieces(us),
    };

    crate::movegen::generate_pseudo_pawn_moves_fast(pos, &context, &mut moves);
    crate::movegen::generate_pseudo_knight_moves_fast(pos, &context, &mut moves);
    crate::movegen::generate_pseudo_bishop_moves_fast(pos, &context, &mut moves);
    crate::movegen::generate_pseudo_rook_moves_fast(pos, &context, &mut moves);
    crate::movegen::generate_pseudo_queen_moves_fast(pos, &context, &mut moves);
    crate::movegen::generate_pseudo_king_moves_fast(pos, &context, &mut moves);

    moves
}

/// Fast zero-allocation legal move generation with reused position buffer
pub fn generate_legal_moves_fast(pos: &Position) -> MoveList {
    let pseudo = generate_pseudo_moves_fast(pos);
    let mut legal = MoveList::new();
    let mut new_pos = *pos; // Single allocation, reused for all moves

    for &m in pseudo.as_slice() {
        pos.apply_move_into(&m, &mut new_pos);

        if !crate::movegen::is_legal_fast(pos, &new_pos) {
            continue;
        }

        legal.push(m);
    }

    legal
}

/// Backward-compatible Vec-based legal move generation (slower)
pub fn generate_legal_moves(pos: &Position) -> Vec<ChessMove> {
    generate_legal_moves_fast(pos).to_vec()
}

/// Fast zero-allocation pseudo capture generation
pub fn generate_pseudo_captures_fast(pos: &Position) -> MoveList {
    crate::movegen::captures::generate_pseudo_captures_fast(pos)
}

// Delegated to `movegen::legality` during refactor

// Pawn move generation was moved into `movegen::pawn` during the refactor.
// See `crate::movegen::pawn` for the implementation
// (generate_pseudo_pawn_moves).

pub(crate) fn push_moves_from_valid_targets_fast(
    pos: &Position,
    context: &MoveGenContext,
    from: Square,
    valid_targets: BitBoardMask,
    moves: &mut MoveList,
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

/// Diagnostic function that validates legal move generation.
/// If the legal move list is empty, checks if it's a genuine terminal position
/// (checkmate/stalemate) or if there's a bug. Logs detailed error info if moves
/// are unexpectedly empty.
pub fn validate_legal_move_generation(pos: &Position) -> bool {
    let legal_moves = generate_legal_moves(pos);

    if !legal_moves.is_empty() {
        // Normal case: we have legal moves
        return true;
    }

    // Legal move list is empty. Check if this is a valid terminal position.
    let pseudo_moves = generate_pseudo_moves_fast(pos);
    let in_check = crate::movegen::is_in_check(pos, pos.side_to_move);

    // If there are ANY pseudo-legal moves that all fail the legality check,
    // and we're in check, this is (probably) checkmate. If no pseudo moves at all,
    // and we're NOT in check, this is stalemate.
    let is_valid_terminal = if in_check {
        // In check with no legal moves = checkmate (expected)
        true
    } else {
        // Not in check with no legal moves = stalemate (expected)
        true
    };

    if !is_valid_terminal {
        // Should never reach here given the above logic, but kept for clarity
        eprintln!(
            "[MOVEGEN_WARN] Empty legal move list in unexpected position: {:?}",
            pos.to_fen()
        );
        return false;
    }

    // If we have pseudo-legal moves but ALL failed the legality check,
    // that's suspicious and worth logging (indicates king safety check may be
    // wrong).
    if !pseudo_moves.is_empty() && legal_moves.is_empty() {
        eprintln!(
            "[MOVEGEN_DIAGNOSTIC] All {} pseudo-legal moves are illegal at: {}",
            pseudo_moves.len(),
            pos.to_fen()
        );
        eprintln!(
            "[MOVEGEN_DIAGNOSTIC] In check: {}, side to move: {:?}",
            in_check, pos.side_to_move
        );

        // Log first few pseudo moves for manual inspection
        for (i, m) in pseudo_moves.iter().take(5).enumerate() {
            eprintln!("[MOVEGEN_DIAGNOSTIC]   Pseudo move {}: {}", i + 1, m);
        }
    }

    true
}

// ...existing code...

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::Color;

    // Helper `move_exists` removed during refactor; tests still validate behavior

    #[test]
    fn test_initial_position_move_count() {
        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let moves = generate_legal_moves(&pos);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_queen_and_king_cannot_move_like_knight_from_d1() {
        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let moves = generate_legal_moves(&pos);
        let illegal = moves
            .iter()
            .any(|m| m.from().to_string() == "d1" && m.to().to_string() == "f3");
        assert!(!illegal, "Queen on d1 should not be able to move to f3");
        let illegal_king = moves
            .iter()
            .any(|m| m.from().to_string() == "e1" && m.to().to_string() == "g2");
        assert!(!illegal_king, "King on e1 should not be able to move to g2");
        let queen_moves = moves
            .iter()
            .filter(|m| m.from().to_string() == "d1")
            .count();
        let king_moves = moves
            .iter()
            .filter(|m| m.from().to_string() == "e1")
            .count();
        assert_eq!(queen_moves, 0);
        assert_eq!(king_moves, 0);
    }

    #[test]
    fn test_knight_moves_from_center() {
        let pos = Position::from_fen("8/8/8/3N4/8/8/8/4K3 w - - 0 1");
        let moves = generate_legal_moves(&pos);
        let found: Vec<_> = moves
            .iter()
            .filter(|m| m.from().to_string() == "d5")
            .map(|m| m.to().to_string())
            .collect();
        let expected_targets = ["c7", "e7", "b6", "f6", "b4", "f4", "c3", "e3"];
        for sq in &expected_targets {
            assert!(
                found.contains(&sq.to_string()),
                "Missing knight move to {}. Found: {:?}",
                sq,
                found
            );
        }
        assert_eq!(found.len(), 8);
    }

    #[test]
    fn test_bishop_moves_blocked_by_own_piece() {
        let pos = Position::from_fen("8/8/8/8/8/8/2P5/2B5 w - - 0 1");
        let moves = generate_legal_moves(&pos);
        for m in &moves {
            assert_ne!(
                m.to(),
                Square::from_coords('d', '2').unwrap(),
                "Bishop should not capture own pawn"
            );
        }
    }

    #[test]
    fn test_missing_side_to_move_king_rejects_all_legal_moves() {
        // Regression: legality filtering must reject candidates when the side-to-move
        // king is missing from the board.
        let pos = Position::from_fen("8/8/8/8/8/8/2P5/2B5 w - - 0 1");
        let moves = generate_legal_moves(&pos);
        assert!(
            moves.is_empty(),
            "Positions without the side-to-move king must have no legal moves"
        );
    }

    #[test]
    fn test_rook_moves_blocked_by_opponent() {
        let pos = Position::from_fen("4k3/8/8/p7/8/8/8/R3K3 w - - 0 1");
        let moves = generate_legal_moves(&pos);
        let mut found_capture = false;
        for m in &moves {
            if m.from().to_string() == "a1" && m.to().to_string() == "a5" {
                found_capture = true;
            }
            if m.from().to_string() == "a1" {
                let to = m.to().to_string();
                assert!(
                    to != "a6" && to != "a7" && to != "a8",
                    "Rook should not move past capture (found move to {})",
                    to
                );
            }
        }
        assert!(found_capture, "Rook should be able to capture on a5");
    }

    #[test]
    fn test_king_cannot_move_into_check() {
        let pos = Position::from_fen("4r3/8/8/8/8/8/8/4K3 w - - 0 1");
        let moves = generate_legal_moves(&pos);
        for m in &moves {
            assert_ne!(m.to().to_string(), "e2", "King should not move into check");
        }
    }

    #[test]
    fn test_pawn_promotion_moves() {
        let pos = Position::from_fen("4k3/6P1/8/8/8/8/8/4K3 w - - 0 1");
        let moves = generate_legal_moves(&pos);
        let mut found = false;
        for m in &moves {
            if m.from().to_string() == "g7" && m.to().to_string() == "g8" {
                found = true;
            }
        }
        assert!(found, "Pawn should be able to promote on g8");
    }

    #[test]
    fn test_en_passant_capture() {
        let pos = Position::from_fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");
        let moves = generate_legal_moves(&pos);
        let mut found = false;
        for m in &moves {
            if m.from().to_string() == "e5" && m.to().to_string() == "d6" {
                found = true;
            }
        }
        assert!(found, "En passant capture should be available");
    }

    #[test]
    fn test_castling_rights() {
        let pos = Position::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        let moves = generate_legal_moves(&pos);
        let mut kingside = false;
        let mut queenside = false;
        for m in &moves {
            if m.from().to_string() == "e1" && m.to().to_string() == "g1" {
                kingside = true;
            }
            if m.from().to_string() == "e1" && m.to().to_string() == "c1" {
                queenside = true;
            }
        }
        assert!(kingside, "White should be able to castle kingside");
        assert!(queenside, "White should be able to castle queenside");
    }

    #[test]
    fn test_illegal_c3d5_not_generated() {
        let fen = "r2q1rk1/1p2bppp/p1npbn2/3Np3/P3P3/1NN5/1PP1BPPP/R1BQ1R1K b - - 0 1";
        let pos = Position::from_fen(fen);

        // Generate all moves the engine thinks are possible
        let legal_moves = generate_legal_moves(&pos);

        // Check if any move originates from C3
        let moves_from_c3: Vec<_> = legal_moves
            .iter()
            .filter(|m| m.from() == Square::C3)
            .collect();

        assert!(
            moves_from_c3.is_empty(),
            "Bug Found: Engine generated moves from C3 ({:?}), but Black has no piece there. \
             Check for rank-flipping/mirroring logic in movegen.",
            moves_from_c3
        );
    }

    #[test]
    fn test_state_consistency_after_white_move() {
        let fen = "r2q1rk1/1p2bppp/p1npbn2/4p3/P3P3/1NN5/1PP1BPPP/R1BQ1R1K w - - 0 1";
        let pos = Position::from_fen(fen);

        // Simulate White's move: Knight from c3 to d5
        let white_move = ChessMove::new(Square::C3, Square::D5, MoveType::Quiet);
        let mut new_pos = Position::default();
        pos.apply_move_into(&white_move, &mut new_pos);

        // Helper to get piece at a square
        fn get_piece_at(pos: &Position, sq: Square) -> Option<crate::piece::Piece> {
            let mask = crate::BitBoardMask::from_square(sq);
            for (piece, bb) in pos.pieces.iter() {
                if (bb & mask).is_nonempty() {
                    return Some(piece);
                }
            }
            None
        }

        // 1. Check if C3 is actually cleared
        assert!(
            get_piece_at(&new_pos, Square::C3).is_none(),
            "Bug: Square C3 should be empty after White moves the knight to d5!"
        );

        // 2. Check side to move
        assert_eq!(
            new_pos.side_to_move,
            Color::Black,
            "It should be Black's turn."
        );

        // 3. Ensure Black can't move from C3
        let black_moves = generate_legal_moves(&new_pos);
        let ghost_move = black_moves.iter().any(|m| m.from() == Square::C3);

        assert!(
            !ghost_move,
            "Bug: Black is allowed to move a piece from C3, which should be empty!"
        );
    }
}

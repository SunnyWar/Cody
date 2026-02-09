/// Minimal perft (performance test) for move generation verification.
/// Perft counts the number of leaf nodes at a given depth from a position.
///
/// Example: perft(position, 1) counts all legal moves from position.
/// Example: perft(position, 3) counts all possible positions 3 moves ahead.
use crate::{mov::ChessMove, movegen::generate_legal_moves, position::Position};

/// Count leaf nodes at the given depth from the given position.
/// Returns the number of possible game paths of that length.
pub fn perft(pos: &Position, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_legal_moves(pos);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut count: u64 = 0;
    let mut next_pos = Position::default();

    for mv in moves {
        pos.apply_move_into(&mv, &mut next_pos);
        count += perft(&next_pos, depth - 1);
    }

    count
}

/// Divide a position into move-by-move leaf counts (useful for debugging).
/// Returns a string with each legal move and its perft(depth-1) count.
pub fn perft_divide(pos: &Position, depth: u32) -> String {
    if depth == 0 {
        return String::new();
    }

    let moves = generate_legal_moves(pos);

    let mut result = String::new();
    let mut next_pos = Position::default();
    let mut total: u64 = 0;

    for mv in moves {
        pos.apply_move_into(&mv, &mut next_pos);
        let count = perft(&next_pos, depth - 1);
        total += count;
        result.push_str(&format!("{}: {}\n", move_to_uci(&mv), count));
    }

    result.push_str(&format!("Total: {}\n", total));
    result
}

/// Convert a ChessMove to UCI notation (e.g., "e2e4").
fn move_to_uci(mv: &ChessMove) -> String {
    let from = format!("{}", mv.from);
    let to = format!("{}", mv.to);

    // Handle promotions
    use crate::mov::MoveType;
    match mv.move_type {
        MoveType::Promotion(kind) => {
            let promotion_char = match kind {
                crate::piece::PieceKind::Knight => 'n',
                crate::piece::PieceKind::Bishop => 'b',
                crate::piece::PieceKind::Rook => 'r',
                crate::piece::PieceKind::Queen => 'q',
                _ => '?',
            };
            format!("{}{}{}", from, to, promotion_char)
        }
        _ => format!("{}{}", from, to),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perft_initial_position() {
        let pos = Position::default();
        assert_eq!(
            perft(&pos, 1),
            20,
            "Initial position should have 20 legal moves"
        );
    }

    #[test]
    fn test_perft_depth_0() {
        let pos = Position::default();
        assert_eq!(perft(&pos, 0), 1, "Perft at depth 0 should always return 1");
    }

    #[test]
    fn test_perft_kiwipete() {
        // Kiwipete position: a complex mid-game position
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 10";
        let pos = Position::from_fen(fen);
        assert_eq!(perft(&pos, 1), 48, "Kiwipete should have 48 legal moves");
    }

    #[test]
    fn test_perft_simple_endgame() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 11";
        let pos = Position::from_fen(fen);
        assert_eq!(
            perft(&pos, 1),
            15,
            "This endgame position should have 15 legal moves"
        );
    }
}

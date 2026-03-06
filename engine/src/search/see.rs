use bitboard::BitBoardMask;
use bitboard::Square;
use bitboard::bitboard::bishop_attacks_from;
use bitboard::bitboard::king_attacks;
use bitboard::bitboard::knight_attacks;
use bitboard::bitboard::pawn_attacks_to;
use bitboard::bitboard::rook_attacks_from;
/// Static Exchange Evaluation (SEE)
///
/// Computes the expected material balance of a capture sequence without doing
/// full alpha-beta search. Used to prune obviously losing captures in
/// quiescence search.
///
/// Algorithm:
/// 1. Start with the moving piece on the target square
/// 2. Find the next least-valuable attacker for the defending side
/// 3. Recursively evaluate what happens if that piece recaptures
/// 4. Stop when no more attackers exist or gain becomes negative
use bitboard::piece::Color;
use bitboard::piece::Piece;
use bitboard::piece::PieceKind;
use bitboard::position::Position;

fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 10000,
    }
}

/// Helper to locate a piece at a square
fn get_piece_at(pos: &Position, sq: Square) -> Option<Piece> {
    let mask = BitBoardMask::from_square(sq);
    for (piece, bb) in pos.pieces.iter() {
        if (bb & mask).is_nonempty() {
            return Some(piece);
        }
    }
    None
}

/// Returns the least-valuable attacker of a given color on a target square.
/// Returns (piece_kind, amount) where amount is how many of that piece type
/// attack.
fn find_least_valuable_attacker(
    pos: &Position,
    target_sq: Square,
    attacking_color: Color,
    occupied: BitBoardMask,
) -> Option<(PieceKind, BitBoardMask)> {
    let pawns = pos
        .pieces
        .get(Piece::from_parts(attacking_color, Some(PieceKind::Pawn)))
        & occupied;
    let pawn_attackers = pawns & pawn_attacks_to(target_sq, attacking_color);
    if pawn_attackers.is_nonempty() {
        return Some((PieceKind::Pawn, pawn_attackers));
    }

    let knights = pos
        .pieces
        .get(Piece::from_parts(attacking_color, Some(PieceKind::Knight)))
        & occupied;
    let knight_attackers = knights & knight_attacks(target_sq);
    if knight_attackers.is_nonempty() {
        return Some((PieceKind::Knight, knight_attackers));
    }

    let bishops = pos
        .pieces
        .get(Piece::from_parts(attacking_color, Some(PieceKind::Bishop)))
        & occupied;
    let bishop_attackers = bishops & bishop_attacks_from(target_sq, occupied);
    if bishop_attackers.is_nonempty() {
        return Some((PieceKind::Bishop, bishop_attackers));
    }

    let rooks = pos
        .pieces
        .get(Piece::from_parts(attacking_color, Some(PieceKind::Rook)))
        & occupied;
    let rook_attackers = rooks & rook_attacks_from(target_sq, occupied);
    if rook_attackers.is_nonempty() {
        return Some((PieceKind::Rook, rook_attackers));
    }

    let queens = pos
        .pieces
        .get(Piece::from_parts(attacking_color, Some(PieceKind::Queen)))
        & occupied;
    let queen_attackers = queens
        & (rook_attacks_from(target_sq, occupied) | bishop_attacks_from(target_sq, occupied));
    if queen_attackers.is_nonempty() {
        return Some((PieceKind::Queen, queen_attackers));
    }

    let kings = pos
        .pieces
        .get(Piece::from_parts(attacking_color, Some(PieceKind::King)))
        & occupied;
    let king_attackers = kings & king_attacks(target_sq);
    if king_attackers.is_nonempty() {
        return Some((PieceKind::King, king_attackers));
    }

    None
}

/// Compute Static Exchange Evaluation for a capture move.
///
/// Returns: the material balance from the perspective of the side making the
/// initial capture. Positive value = good capture, Negative value = bad capture
///
/// Example:
/// - Knight captures Pawn, Pawn recaptures: +100 - 320 = -220 (bad)
/// - Pawn captures Pawn, nothing recaptures: +100 (good)
/// - Queen captures Knight, Pawn recaptures: +320 - 900 = -580 (bad)
pub fn compute_see(pos: &Position, from: Square, to: Square) -> i32 {
    // Get the moving piece
    let moving_piece = match get_piece_at(pos, from) {
        Some(p) => p,
        None => return 0, // No piece at source
    };

    let attacker_color = moving_piece.color();
    let defender_color = attacker_color.opposite();

    // Get the target piece being captured
    let target_piece = match get_piece_at(pos, to) {
        Some(p) => p,
        None => return 0, // No piece to capture
    };

    let gain = piece_value(target_piece.kind());
    let occupied =
        pos.all_pieces() & !BitBoardMask::from_square(from) & !BitBoardMask::from_square(to);
    let occupied = occupied | BitBoardMask::from_square(to); // Moving piece is now on target

    // Opponent's best recapture sequence against the moved piece.
    let opponent_gain = see_recursive(
        pos,
        to,
        defender_color,
        occupied,
        piece_value(moving_piece.kind()),
        0,
    );

    gain - opponent_gain
}

fn see_recursive(
    pos: &Position,
    target_sq: Square,
    defending_color: Color,
    mut occupied: BitBoardMask,
    captured_value: i32,
    depth: u8,
) -> i32 {
    // Exchange length is bounded by total pieces; cap recursion defensively.
    if depth >= 32 {
        return 0;
    }

    // Find the least valuable piece of defending_color that can attack target_sq
    if let Some((piece_kind, piece_bb)) =
        find_least_valuable_attacker(pos, target_sq, defending_color, occupied)
    {
        let piece_sq = match piece_bb.first_square() {
            Some(sq) => sq,
            None => return 0,
        };

        // Remove the attacking piece from occupied
        occupied &= !BitBoardMask::from_square(piece_sq);

        let defender_value = piece_value(piece_kind);
        // If we capture now, opponent may continue the exchange optimally.
        let continuation = see_recursive(
            pos,
            target_sq,
            defending_color.opposite(),
            occupied,
            defender_value,
            depth + 1,
        );

        let gain = captured_value - continuation;
        gain.max(0)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_see_simple_pawn_capture_nothing() {
        // White pawn captures black pawn, nothing recaptures
        let pos = Position::from_fen("k7/8/8/1p6/1P6/8/K7 w - - 0 1");
        let white_pawn_src = Square::B4;
        let black_pawn_dst = Square::B5;

        let see = compute_see(&pos, white_pawn_src, black_pawn_dst);
        assert_eq!(see, 100); // Gain the black pawn, no recapture
    }

    #[test]
    fn test_see_bad_knight_trade() {
        // White queen captures black pawn, rook recaptures queen.
        // Position: kr6/8/8/1p6/1Q6/8/K7/8 w - - 0 1
        // - Black pawn on b5, black rook on b8
        // - White queen on b4 captures black pawn on b5 (Qxb5), gains 100
        // - Black rook on b8 recaptures queen (Rxb5), loses 900
        // - Net for white: 100 - 900 = -800
        let pos = Position::from_fen("kr6/8/8/1p6/1Q6/8/K7/8 w - - 0 1");
        let queen_src = Square::B4;
        let pawn_dst = Square::B5;

        let see = compute_see(&pos, queen_src, pawn_dst);
        assert!(
            see < 0,
            "SEE should be negative for losing exchange, got {}",
            see
        );
    }
}

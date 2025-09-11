/// Offsets a knight can move: (rank_delta, file_delta)
const KNIGHT_DELTAS: [(i8, i8); 8] = [
    ( 2,  1), ( 2, -1),
    (-2,  1), (-2, -1),
    ( 1,  2), ( 1, -2),
    (-1,  2), (-1, -2),
];

/// 64‐bit bitboard wrapper
#[derive(Clone, Copy)]
struct Bitboard(u64);

impl Bitboard {
    const ZERO: Self = Bitboard(0);

    /// Set the bit at this square (0..63)
    const fn with_bit(mut self, sq: u8) -> Self {
        self.0 |= 1u64 << sq;
        self
    }
}

/// Compute knight attacks for a single square
const fn knight_attacks_from(sq: u8) -> Bitboard {
    let rank = (sq / 8) as i8;
    let file = (sq % 8) as i8;
    let mut mask = Bitboard::ZERO;
    let mut i = 0;

    // Iterate over all 8 knight‐deltas
    while i < KNIGHT_DELTAS.len() {
        let (dr, df) = KNIGHT_DELTAS[i];
        let r = rank + dr;
        let f = file + df;

        // Replace `Range::contains` with explicit bounds checks
        if r >= 0 && r < 8 && f >= 0 && f < 8 {
            let dest = (r as u8) * 8 + (f as u8);
            mask = mask.with_bit(dest);
        }
        i += 1;
    }

    mask
}

/// Build the 64‐entry knight‐attack table at compile time
const KNIGHT_ATTACKS: [Bitboard; 64] = {
    let mut table = [Bitboard::ZERO; 64];
    let mut sq = 0;
    while sq < 64 {
        table[sq] = knight_attacks_from(sq as u8);
        sq += 1;
    }
    table
};


/* fn generate_all_knight_moves(
    pos: &Position,
    context: &MoveGenContext,
    moves: &mut Vec<Move>,
) {
    let knights = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Knight)));

    for from in knights.squares() {
        // Table lookup + mask out our own pieces
        let attacks = KNIGHT_ATTACKS[from as usize].and(context.not_ours);
        for to in attacks.squares() {
            moves.push(Move::new(from, to));
        }
    }
}
 */
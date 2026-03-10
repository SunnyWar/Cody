// Simple Zobrist hashing constants for positions.
// For determinism we use a fixed set of precomputed constants derived from a
// small LCG. This keeps the bitboard crate allocation-free and dependency-free.
use crate::piece::Color;
use crate::piece::Piece;

pub const ZOBRIST_PIECE_KEYS: [[u64; 64]; 12] = {
    // Generated with a tiny LCG; values are precomputed offline and embedded here.
    // For brevity in this MVP we use a simple pattern; it's fine for correctness
    // and testing.
    const fn make() -> [[u64; 64]; 12] {
        let mut tbl = [[0u64; 64]; 12];
        let mut i = 0;
        let mut seed: u64 = 0x9e37_79b9_7f4a_7c15u64;
        while i < 12 {
            let mut j = 0;
            while j < 64 {
                seed = seed
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                tbl[i][j] = seed ^ (seed >> 32);
                j += 1;
            }
            i += 1;
        }
        tbl
    }
    make()
};

pub const ZOBRIST_SIDE: u64 = 0xF0E1_D2C3_B4A5_9687u64;
pub const ZOBRIST_CASTLE_KEYS: [u64; 4] = [
    0x0123_4567_89AB_CDEFu64,
    0xFEDC_BA98_7654_3210u64,
    0x0F1E_2D3C_4B5A_6978u64,
    0x89AB_CDEF_0123_4567u64,
];

/// Const lookup table mapping `Piece` discriminant directly to zobrist index.
/// Pieces 0-11 (`WhitePawn`..`BlackKing`) map to zobrist indices 0-11.
/// `Piece::None` (12) maps to 0 (unused in zobrist computation).
const PIECE_ZOBRIST_INDEX: [usize; 13] = [
    0,  // WhitePawn
    1,  // WhiteKnight
    2,  // WhiteBishop
    3,  // WhiteRook
    4,  // WhiteQueen
    5,  // WhiteKing
    6,  // BlackPawn
    7,  // BlackKnight
    8,  // BlackBishop
    9,  // BlackRook
    10, // BlackQueen
    11, // BlackKing
    0,  // None (safe default, unused)
];

/// Map a piece to its zobrist hash table index. Uses const lookup table for
/// O(1) access without branches; suitable for const evaluation.
pub const fn piece_index(p: Piece) -> usize {
    PIECE_ZOBRIST_INDEX[p as usize]
}

use crate::position::Position;

pub fn compute_zobrist(pos: &Position) -> u64 {
    let mut h: u64 = 0;

    for (piece, bb) in pos.pieces.iter() {
        let idx = piece_index(piece);
        // iterate squares set
        for sq in bb.squares() {
            let sqi = sq.index();
            h ^= ZOBRIST_PIECE_KEYS[idx][sqi];
        }
    }

    if pos.side_to_move == Color::Black {
        h ^= ZOBRIST_SIDE;
    }

    // Castling rights: four bits (WK, WQ, BK, BQ)
    if pos.castling_rights.kingside(Color::White) {
        h ^= ZOBRIST_CASTLE_KEYS[0];
    }
    if pos.castling_rights.queenside(Color::White) {
        h ^= ZOBRIST_CASTLE_KEYS[1];
    }
    if pos.castling_rights.kingside(Color::Black) {
        h ^= ZOBRIST_CASTLE_KEYS[2];
    }
    if pos.castling_rights.queenside(Color::Black) {
        h ^= ZOBRIST_CASTLE_KEYS[3];
    }

    // En-passant: include square index when present
    if let Some(sq) = pos.ep_square {
        h ^= ZOBRIST_PIECE_KEYS[0][sq.index()];
    }

    h
}

// Simple Zobrist hashing constants for positions.
// For determinism we use a fixed set of precomputed constants derived from a
// small LCG. This keeps the bitboard crate allocation-free and dependency-free.
use crate::piece::Color;
use crate::piece::Piece;
use crate::piece::PieceKind;

pub const ZOBRIST_PIECE_KEYS: [[u64; 64]; 12] = {
    // Generated with a tiny LCG; values are precomputed offline and embedded here.
    // For brevity in this MVP we use a simple pattern; it's fine for correctness
    // and testing.
    const fn make() -> [[u64; 64]; 12] {
        let mut tbl = [[0u64; 64]; 12];
        let mut i = 0;
        let mut seed: u64 = 0x9e3779b97f4a7c15u64;
        while i < 12 {
            let mut j = 0;
            while j < 64 {
                seed = seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                tbl[i][j] = seed ^ (seed >> 32);
                j += 1;
            }
            i += 1;
        }
        tbl
    }
    make()
};

pub const ZOBRIST_SIDE: u64 = 0xF0E1D2C3B4A59687u64;
pub const ZOBRIST_CASTLE_KEYS: [u64; 4] = [
    0x0123456789ABCDEFu64,
    0xFEDCBA9876543210u64,
    0x0F1E2D3C4B5A6978u64,
    0x89ABCDEF01234567u64,
];

pub fn piece_index(p: Piece) -> usize {
    // map (color, kind) to 0..12
    let color_idx = match p.color() {
        Color::White => 0usize,
        Color::Black => 1usize,
    };
    let kind_idx = match p.kind() {
        PieceKind::Pawn => 0,
        PieceKind::Knight => 1,
        PieceKind::Bishop => 2,
        PieceKind::Rook => 3,
        PieceKind::Queen => 4,
        PieceKind::King => 5,
    };
    color_idx * 6 + kind_idx
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

// src/search/movegen.rs

use crate::core::bitboard::{FILE_A, FILE_H, bit_iter, knight_attacks};
use crate::core::mov::Move;
use crate::core::position::Position;

pub struct SimpleMoveGen;

pub trait MoveGenerator {
    fn generate_moves(&self, pos: &Position) -> Vec<Move>;
    fn in_check(&self, pos: &Position) -> bool;
}

impl MoveGenerator for SimpleMoveGen {
    fn generate_moves(&self, pos: &Position) -> Vec<Move> {
        let mut moves = Vec::new();
        let us = pos.side_to_move;

        self.gen_pawn_moves(pos, us, &mut moves);
        self.gen_knight_moves(pos, us, &mut moves);
        self.gen_bishop_moves(pos, us, &mut moves);
        self.gen_rook_moves(pos, us, &mut moves);
        self.gen_queen_moves(pos, us, &mut moves);
        self.gen_king_moves(pos, us, &mut moves);

        moves
    }

    fn in_check(&self, pos: &Position) -> bool {
        let us = pos.side_to_move;
        let them = 1 - us;

        let king_bb = pos.pieces[if us == 0 { 5 } else { 11 }];
        if king_bb == 0 {
            return false;
        }
        let king_sq = king_bb.trailing_zeros() as u8;
        let occ = pos.all_pieces();

        // 1. Rook/Queen (straight lines)
        let rook_like =
            pos.pieces[if them == 0 { 3 } else { 9 }] | pos.pieces[if them == 0 { 4 } else { 10 }];
        if rook_attacks(king_sq, occ) & rook_like != 0 {
            return true;
        }

        // 2. Bishop/Queen (diagonals)
        let bishop_like =
            pos.pieces[if them == 0 { 2 } else { 8 }] | pos.pieces[if them == 0 { 4 } else { 10 }];
        if bishop_attacks(king_sq, occ) & bishop_like != 0 {
            return true;
        }

        // 3. Knights
        if knight_attacks(king_sq) & pos.pieces[if them == 0 { 1 } else { 7 }] != 0 {
            return true;
        }

        // 4. Pawns
        let pawn_attacks = if them == 0 {
            ((king_bb >> 7) & !FILE_A) | ((king_bb >> 9) & !FILE_H)
        } else {
            ((king_bb << 7) & !FILE_H) | ((king_bb << 9) & !FILE_A)
        };
        if pawn_attacks & pos.pieces[if them == 0 { 0 } else { 6 }] != 0 {
            return true;
        }

        // 5. Opponent king
        if king_attacks(king_sq) & pos.pieces[if them == 0 { 5 } else { 11 }] != 0 {
            return true;
        }

        false
    }
}

impl SimpleMoveGen {
    fn gen_pawn_moves(&self, pos: &Position, us: u8, moves: &mut Vec<Move>) {
        let pawns = pos.pieces[if us == 0 { 0 } else { 6 }];
        let empty = !pos.all_pieces();
        let their_pieces = pos.their_pieces(us);

        if us == 0 {
            // single pushes
            let single_push = (pawns << 8) & empty;
            for to in bit_iter(single_push) {
                moves.push(Move::new(to - 8, to));
            }
            // double pushes
            let rank4_mask: u64 = 0x00000000FF000000; // adjust for your indexing
            let double_push = ((single_push << 8) & empty) & rank4_mask;
            for to in bit_iter(double_push) {
                moves.push(Move::new(to - 16, to));
            }
            // captures
            let left_caps = (pawns << 7) & their_pieces & !FILE_H;
            let right_caps = (pawns << 9) & their_pieces & !FILE_A;
            for to in bit_iter(left_caps) {
                moves.push(Move::new(to - 7, to));
            }
            for to in bit_iter(right_caps) {
                moves.push(Move::new(to - 9, to));
            }
        } else {
            // single pushes
            let single_push = (pawns >> 8) & empty;
            for to in bit_iter(single_push) {
                moves.push(Move::new(to + 8, to));
            }
            // double pushes
            let rank5_mask: u64 = 0x000000FF00000000; // adjust for your indexing
            let double_push = ((single_push >> 8) & empty) & rank5_mask;
            for to in bit_iter(double_push) {
                moves.push(Move::new(to + 16, to));
            }
            // captures
            let left_caps = (pawns >> 9) & their_pieces & !FILE_H;
            let right_caps = (pawns >> 7) & their_pieces & !FILE_A;
            for to in bit_iter(left_caps) {
                moves.push(Move::new(to + 9, to));
            }
            for to in bit_iter(right_caps) {
                moves.push(Move::new(to + 7, to));
            }
        }
    }

    fn gen_knight_moves(&self, pos: &Position, us: u8, moves: &mut Vec<Move>) {
        let knights = pos.pieces[if us == 0 { 1 } else { 7 }];
        let targets = !pos.our_pieces(us);
        for from in bit_iter(knights) {
            let attacks = knight_attacks(from) & targets;
            for to in bit_iter(attacks) {
                moves.push(Move::new(from, to));
            }
        }
    }

    fn gen_bishop_moves(&self, pos: &Position, us: u8, moves: &mut Vec<Move>) {
        let bishops = pos.pieces[if us == 0 { 2 } else { 8 }];
        let targets = !pos.our_pieces(us);
        for from in bit_iter(bishops) {
            let attacks = bishop_attacks(from, pos.all_pieces()) & targets;
            for to in bit_iter(attacks) {
                moves.push(Move::new(from, to));
            }
        }
    }

    fn gen_rook_moves(&self, pos: &Position, us: u8, moves: &mut Vec<Move>) {
        let rooks = pos.pieces[if us == 0 { 3 } else { 9 }];
        let targets = !pos.our_pieces(us);
        for from in bit_iter(rooks) {
            let attacks = rook_attacks(from, pos.all_pieces()) & targets;
            for to in bit_iter(attacks) {
                moves.push(Move::new(from, to));
            }
        }
    }

    fn gen_queen_moves(&self, pos: &Position, us: u8, moves: &mut Vec<Move>) {
        let queens = pos.pieces[if us == 0 { 4 } else { 10 }];
        let targets = !pos.our_pieces(us);
        for from in bit_iter(queens) {
            let attacks = queen_attacks(from, pos.all_pieces()) & targets;
            for to in bit_iter(attacks) {
                moves.push(Move::new(from, to));
            }
        }
    }

    fn gen_king_moves(&self, pos: &Position, us: u8, moves: &mut Vec<Move>) {
        let king = pos.pieces[if us == 0 { 5 } else { 11 }];
        let targets = !pos.our_pieces(us);
        for from in bit_iter(king) {
            let attacks = king_attacks(from) & targets;
            for to in bit_iter(attacks) {
                moves.push(Move::new(from, to));
            }
        }
        // Castling can be added here
    }
}

pub fn rook_attacks(sq: u8, occ: u64) -> u64 {
    let mut attacks = 0u64;
    let mut mask;

    // North
    mask = sq as i8 + 8;
    while mask < 64 {
        attacks |= 1u64 << mask;
        if occ & (1u64 << mask) != 0 {
            break;
        }
        mask += 8;
    }

    // South
    mask = sq as i8 - 8;
    while mask >= 0 {
        attacks |= 1u64 << mask;
        if occ & (1u64 << mask) != 0 {
            break;
        }
        mask -= 8;
    }

    // East
    mask = sq as i8 + 1;
    while mask % 8 != 0 {
        attacks |= 1u64 << mask;
        if occ & (1u64 << mask) != 0 {
            break;
        }
        mask += 1;
    }

    // West
    mask = sq as i8 - 1;
    while mask % 8 != 7 && mask >= 0 {
        attacks |= 1u64 << mask;
        if occ & (1u64 << mask) != 0 {
            break;
        }
        mask -= 1;
    }

    attacks
}

pub fn queen_attacks(sq: u8, occ: u64) -> u64 {
    rook_attacks(sq, occ) | bishop_attacks(sq, occ)
}

pub fn king_attacks(sq: u8) -> u64 {
    let bb = 1u64 << sq;
    let not_a = 0xfefefefefefefefe;
    let not_h = 0x7f7f7f7f7f7f7f7f;

    let mut attacks = 0u64;
    attacks |= (bb << 8) | (bb >> 8); // up/down
    attacks |= (bb << 1) & not_a; // right
    attacks |= (bb >> 1) & not_h; // left
    attacks |= (bb << 9) & not_a; // up-right
    attacks |= (bb << 7) & not_h; // up-left
    attacks |= (bb >> 7) & not_a; // down-right
    attacks |= (bb >> 9) & not_h; // down-left
    attacks
}

pub fn bishop_attacks(sq: u8, occ: u64) -> u64 {
    let mut attacks = 0u64;
    let mut mask;

    // NE
    mask = sq as i8 + 9;
    while mask < 64 && mask % 8 != 0 {
        attacks |= 1u64 << mask;
        if occ & (1u64 << mask) != 0 {
            break;
        }
        mask += 9;
    }

    // NW
    mask = sq as i8 + 7;
    while mask < 64 && mask % 8 != 7 {
        attacks |= 1u64 << mask;
        if occ & (1u64 << mask) != 0 {
            break;
        }
        mask += 7;
    }

    // SE
    mask = sq as i8 - 7;
    while mask >= 0 && mask % 8 != 0 {
        attacks |= 1u64 << mask;
        if occ & (1u64 << mask) != 0 {
            break;
        }
        mask -= 7;
    }

    // SW
    mask = sq as i8 - 9;
    while mask >= 0 && mask % 8 != 7 {
        attacks |= 1u64 << mask;
        if occ & (1u64 << mask) != 0 {
            break;
        }
        mask -= 9;
    }

    attacks
}

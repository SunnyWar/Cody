// src/search/movegen.rs

use crate::core::bitboard::{
    BISHOP_ATTACKS, BISHOP_MASKS, FILE_A, FILE_H, KING_ATTACKS, KNIGHT_ATTACKS, ROOK_ATTACKS,
    ROOK_MASKS, bit_iter, occ_to_index,
};
use crate::core::mov::Move;
use crate::core::piece::{Color, PieceType, piece_index};
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
        let us: Color = pos.side_to_move;
        let them = match us {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        let king_bb = pos.pieces[piece_index(us, PieceType::King)];
        if king_bb == 0 {
            return false;
        }
        let king_sq = king_bb.trailing_zeros() as usize;
        let occ = pos.all_pieces();

        // 1. Rook/Queen (straight lines)
        let rook_like = pos.pieces[piece_index(them, PieceType::Rook)]
            | pos.pieces[piece_index(them, PieceType::Queen)];
        let rmask = ROOK_MASKS[king_sq];
        let rindex = occ_to_index(occ & rmask, rmask);
        if ROOK_ATTACKS[king_sq][rindex] & rook_like != 0 {
            return true;
        }

        // 2. Bishop/Queen (diagonals)
        let bishop_like = pos.pieces[piece_index(them, PieceType::Bishop)]
            | pos.pieces[piece_index(them, PieceType::Queen)];
        let bmask = BISHOP_MASKS[king_sq];
        let bindex = occ_to_index(occ & bmask, bmask);
        if BISHOP_ATTACKS[king_sq][bindex] & bishop_like != 0 {
            return true;
        }

        // 3. Knights
        if KNIGHT_ATTACKS[king_sq] & pos.pieces[piece_index(them, PieceType::Knight)] != 0 {
            return true;
        }

        // 4. Pawns
        let pawn_attacks = match them {
            Color::White => ((king_bb >> 7) & !FILE_A) | ((king_bb >> 9) & !FILE_H),
            Color::Black => ((king_bb << 7) & !FILE_H) | ((king_bb << 9) & !FILE_A),
        };
        if pawn_attacks & pos.pieces[piece_index(them, PieceType::Pawn)] != 0 {
            return true;
        }

        // 5. Opponent king
        if KING_ATTACKS[king_sq] & pos.pieces[piece_index(them, PieceType::King)] != 0 {
            return true;
        }

        false
    }
}

impl SimpleMoveGen {
    fn gen_pawn_moves(&self, pos: &Position, us: Color, moves: &mut Vec<Move>) {
        let pawns = pos.pieces[if us == Color::White { 0 } else { 6 }];
        let empty = !pos.all_pieces();
        let their_pieces = pos.their_pieces(us);

        if us == Color::White {
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

    fn gen_knight_moves(&self, pos: &Position, us: Color, moves: &mut Vec<Move>) {
        let knights = pos.pieces[if us == Color::White { 1 } else { 7 }];
        let targets = !pos.our_pieces(us);
        for from in bit_iter(knights) {
            let attacks = KNIGHT_ATTACKS[from as usize] & targets;
            for to in bit_iter(attacks) {
                moves.push(Move::new(from, to));
            }
        }
    }

    fn gen_bishop_moves(&self, pos: &Position, us: Color, moves: &mut Vec<Move>) {
        let bishops = pos.pieces[if us == Color::White { 2 } else { 8 }];
        let targets = !pos.our_pieces(us);
        let occ = pos.all_pieces();

        for from in bit_iter(bishops) {
            let mask = BISHOP_MASKS[from as usize];
            let index = occ_to_index(occ & mask, mask);
            let attacks = BISHOP_ATTACKS[from as usize][index] & targets;

            for to in bit_iter(attacks) {
                moves.push(Move::new(from, to));
            }
        }
    }

    fn gen_rook_moves(&self, pos: &Position, us: Color, moves: &mut Vec<Move>) {
        let rooks = pos.pieces[if us == Color::White { 3 } else { 9 }];
        let targets = !pos.our_pieces(us);
        let occ = pos.all_pieces();

        for from in bit_iter(rooks) {
            let mask = ROOK_MASKS[from as usize];
            let index = occ_to_index(occ & mask, mask);
            let attacks = ROOK_ATTACKS[from as usize][index] & targets;

            for to in bit_iter(attacks) {
                moves.push(Move::new(from, to));
            }
        }
    }

    fn gen_queen_moves(&self, pos: &Position, us: Color, moves: &mut Vec<Move>) {
        let queens = pos.pieces[if us == Color::White { 4 } else { 10 }];
        let targets = !pos.our_pieces(us);
        let occ = pos.all_pieces();

        for from in bit_iter(queens) {
            // Rook component
            let rmask = ROOK_MASKS[from as usize];
            let rindex = occ_to_index(occ & rmask, rmask);
            let rook_attacks = ROOK_ATTACKS[from as usize][rindex];

            // Bishop component
            let bmask = BISHOP_MASKS[from as usize];
            let bindex = occ_to_index(occ & bmask, bmask);
            let bishop_attacks = BISHOP_ATTACKS[from as usize][bindex];

            // Combine and mask with targets
            let attacks = (rook_attacks | bishop_attacks) & targets;

            for to in bit_iter(attacks) {
                moves.push(Move::new(from, to));
            }
        }
    }

    fn gen_king_moves(&self, pos: &Position, us: Color, moves: &mut Vec<Move>) {
        let king = pos.pieces[if us == Color::White { 5 } else { 11 }];
        let targets = !pos.our_pieces(us);

        for from in bit_iter(king) {
            let attacks = KING_ATTACKS[from as usize] & targets;
            for to in bit_iter(attacks) {
                moves.push(Move::new(from, to));
            }
        }

        // TODO: Castling logic can be added here if desired
    }
}

// src/search/movegen.rs

use crate::core::bitboard::{
    ANTIDIAGONAL_MASKS, BISHOP_ATTACKS, BISHOP_MASKS, DIAGONAL_MASKS, DOUBLE_NORTH, DOUBLE_SOUTH,
    FILE_A, FILE_H, FILE_MASKS, KING_ATTACKS, KNIGHT_ATTACKS, NORTH, NORTH_EAST, NORTH_WEST,
    PAWN_ATTACKS, RANK_4_MASK, RANK_5_MASK, RANK_MASKS, ROOK_ATTACKS, ROOK_MASKS, SOUTH,
    SOUTH_EAST, SOUTH_WEST, SQUARE_COLOR_MASK, bit_iter, occ_to_index,
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
        let us = pos.side_to_move;
        let them = us.opposite();

        let king_bb = pos.pieces[piece_index(us, PieceType::King)];
        if king_bb == 0 {
            return false;
        }

        let king_sq = king_bb.trailing_zeros() as usize;
        let occ = pos.occupancy[2];
        let king_color_mask = SQUARE_COLOR_MASK[king_sq];

        // 1. Knights (opposite color only)
        let knight_like = pos.pieces[piece_index(them, PieceType::Knight)] & !king_color_mask;
        if knight_like != 0 && (KNIGHT_ATTACKS[king_sq] & knight_like) != 0 {
            return true;
        }

        // 2. Pawns (same color only)
        let pawn_like = pos.pieces[piece_index(them, PieceType::Pawn)] & king_color_mask;
        if pawn_like != 0 && (PAWN_ATTACKS[them as usize][king_sq] & pawn_like) != 0 {
            return true;
        }

        // 3. Opponent king
        let opp_king = pos.pieces[piece_index(them, PieceType::King)];
        if opp_king != 0 && (KING_ATTACKS[king_sq] & opp_king) != 0 {
            return true;
        }

        // 4. Rook/Queen (same rank/file only)
        let rook_like = (pos.pieces[piece_index(them, PieceType::Rook)]
            | pos.pieces[piece_index(them, PieceType::Queen)])
            & (RANK_MASKS[king_sq] | FILE_MASKS[king_sq]);
        if rook_like != 0 {
            let rmask = ROOK_MASKS[king_sq];
            let rindex = occ_to_index(occ & rmask, rmask);
            if (ROOK_ATTACKS[king_sq][rindex] & rook_like) != 0 {
                return true;
            }
        }

        // 5. Bishop/Queen (same color + same diagonal/antidiagonal)
        let bishop_like = (pos.pieces[piece_index(them, PieceType::Bishop)]
            | pos.pieces[piece_index(them, PieceType::Queen)])
            & king_color_mask
            & (DIAGONAL_MASKS[king_sq] | ANTIDIAGONAL_MASKS[king_sq]);
        if bishop_like != 0 {
            let bmask = BISHOP_MASKS[king_sq];
            let bindex = occ_to_index(occ & bmask, bmask);
            if (BISHOP_ATTACKS[king_sq][bindex] & bishop_like) != 0 {
                return true;
            }
        }

        false
    }
}

impl SimpleMoveGen {
    fn gen_pawn_moves(&self, pos: &Position, us: Color, moves: &mut Vec<Move>) {
        let pawns = pos.pieces[piece_index(us, PieceType::Pawn)];
        if pawns == 0 {
            return; // no pawns, nothing to do
        }

        let empty = !pos.all_pieces();
        let their_pieces = pos.their_pieces(us);

        if us == Color::White {
            let single_push = (pawns << NORTH) & empty;
            if single_push != 0 {
                for to in bit_iter(single_push) {
                    moves.push(Move::new((to as i32 - NORTH) as u8, to));
                }
            }

            let double_push = ((single_push << NORTH) & empty) & RANK_4_MASK;
            if double_push != 0 {
                for to in bit_iter(double_push) {
                    moves.push(Move::new((to as i32 - DOUBLE_NORTH) as u8, to));
                }
            }

            let left_caps = (pawns << NORTH_WEST) & their_pieces & !FILE_H;
            if left_caps != 0 {
                for to in bit_iter(left_caps) {
                    moves.push(Move::new((to as i32 - NORTH_WEST) as u8, to));
                }
            }

            let right_caps = (pawns << NORTH_EAST) & their_pieces & !FILE_A;
            if right_caps != 0 {
                for to in bit_iter(right_caps) {
                    moves.push(Move::new((to as i32 - NORTH_EAST) as u8, to));
                }
            }
        } else {
            let single_push = (pawns >> -SOUTH) & empty;
            if single_push != 0 {
                for to in bit_iter(single_push) {
                    moves.push(Move::new((to as i32 - SOUTH) as u8, to));
                }
            }

            let double_push = ((single_push >> -SOUTH) & empty) & RANK_5_MASK;
            if double_push != 0 {
                for to in bit_iter(double_push) {
                    moves.push(Move::new((to as i32 - DOUBLE_SOUTH) as u8, to));
                }
            }

            let left_caps = (pawns >> -SOUTH_EAST) & their_pieces & !FILE_H;
            if left_caps != 0 {
                for to in bit_iter(left_caps) {
                    moves.push(Move::new((to as i32 - SOUTH_EAST) as u8, to));
                }
            }

            let right_caps = (pawns >> -SOUTH_WEST) & their_pieces & !FILE_A;
            if right_caps != 0 {
                for to in bit_iter(right_caps) {
                    moves.push(Move::new((to as i32 - SOUTH_WEST) as u8, to));
                }
            }
        }
    }

    fn gen_knight_moves(&self, pos: &Position, us: Color, moves: &mut Vec<Move>) {
        let knights = pos.pieces[piece_index(us, PieceType::Knight)];
        let targets = pos.their_pieces(us);
        for from in bit_iter(knights) {
            let attacks = KNIGHT_ATTACKS[from as usize] & targets;
            for to in bit_iter(attacks) {
                moves.push(Move::new(from, to));
            }
        }
    }

    fn gen_bishop_moves(&self, pos: &Position, us: Color, moves: &mut Vec<Move>) {
        let bishops = pos.pieces[piece_index(us, PieceType::Bishop)];
        let targets = pos.their_pieces(us);
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
        let rooks = pos.pieces[piece_index(us, PieceType::Rook)];
        let targets = pos.their_pieces(us);
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
        let queens = pos.pieces[piece_index(us, PieceType::Queen)];
        let targets = pos.their_pieces(us);
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
        let king = pos.pieces[piece_index(us, PieceType::King)];
        let targets = pos.their_pieces(us);

        for from in bit_iter(king) {
            let attacks = KING_ATTACKS[from as usize] & targets;
            for to in bit_iter(attacks) {
                moves.push(Move::new(from, to));
            }
        }

        // TODO: Castling logic can be added here if desired
    }
}

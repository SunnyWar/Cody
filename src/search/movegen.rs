// src/search/movegen.rs
use crate::core::position::{Position};
use crate::search::traits::MoveGenerator;
use crate::core::bitboard::{bit_iter, knight_attacks};
use crate::core::mov::{Move};

pub struct SimpleMoveGen;

impl MoveGenerator for SimpleMoveGen {
    fn generate_moves(&self, pos: &Position) -> Vec<Move> {
        let mut moves = Vec::new();

        let us = pos.side_to_move;
        let pawns = pos.pieces[if us == 0 { 0 } else { 6 }];
        let knights = pos.pieces[if us == 0 { 1 } else { 7 }];
        let empty = !pos.all_pieces();

        // Pawn pushes (single only, no promotions yet)
        if us == 0 {
            let single_push = (pawns << 8) & empty;
            for to in bit_iter(single_push) {
                moves.push(Move::new(to - 8, to));
            }
        } else {
            let single_push = (pawns >> 8) & empty;
            for to in bit_iter(single_push) {
                moves.push(Move::new(to + 8, to));
            }
        }

        // Knight moves (no legality check)
        for from in bit_iter(knights) {
            let attacks = knight_attacks(from) & !pos.our_pieces(us);
            for to in bit_iter(attacks) {
                moves.push(Move::new(from, to));
            }
        }

        moves
    }
}


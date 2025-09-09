// src/search/movegen.rs

use crate::core::bitboard::{
    ANTIDIAGONAL_MASKS, BISHOP_ATTACKS, BISHOP_MASKS, DIAGONAL_MASKS, FILE_A, FILE_H, FILE_MASKS,
    KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, RANK_MASKS, ROOK_ATTACKS, ROOK_MASKS,
    SQUARE_COLOR_MASK, occ_to_index,
};
use crate::core::bitboardmask::{
    BitBoardMask, bishop_attacks_mask, king_attacks_mask, knight_attacks_mask, rook_attacks_mask,
};
use crate::core::mov::Move;
use crate::core::piece::{Color, Piece, PieceKind};
use crate::core::position::{OccupancyKind, Position};

const NORTH: i8 = 8;
const SOUTH: i8 = -8;
const NORTH_EAST: i8 = 9;
const NORTH_WEST: i8 = 7;
const SOUTH_EAST: i8 = -7;
const SOUTH_WEST: i8 = -9;
const DOUBLE_NORTH: i8 = 16;
const DOUBLE_SOUTH: i8 = -16;

//const RANK_1_MASK: u64 = 0x00000000000000FF;
//const RANK_2_MASK: u64 = 0x000000000000FF00;
//const RANK_3_MASK: u64 = 0x0000000000FF0000;
const RANK_4_MASK: u64 = 0x00000000FF000000;
const RANK_5_MASK: u64 = 0x000000FF00000000;
//const RANK_6_MASK: u64 = 0x0000FF0000000000;
//const RANK_7_MASK: u64 = 0x00FF000000000000;
//const RANK_8_MASK: u64 = 0xFF00000000000000;

pub struct SimpleMoveGen;

pub trait MoveGenerator {
    fn generate_moves(&self, pos: &Position) -> Vec<Move>;
    fn in_check(&self, pos: &Position) -> bool;
}

struct MoveGenContext {
    us: Color,
    occ: BitBoardMask,
    not_ours: BitBoardMask,
}

impl MoveGenerator for SimpleMoveGen {
    fn generate_moves(&self, pos: &Position) -> Vec<Move> {
        let mut moves = Vec::new();
        let ctx = MoveGenContext {
            us: pos.side_to_move,
            occ: pos.all_pieces(),
            not_ours: !pos.our_pieces(pos.side_to_move),
        };

        self.generate_all_pawn_moves(pos, &ctx, &mut moves);
        self.generate_all_knight_moves(pos, &ctx, &mut moves);
        self.generate_all_bishop_moves(pos, &ctx, &mut moves);
        self.generate_all_rook_moves(pos, &ctx, &mut moves);
        self.generate_all_queen_moves(pos, &ctx, &mut moves);
        self.generate_all_king_moves(pos, &ctx, &mut moves);

        moves
    }

    fn in_check(&self, pos: &Position) -> bool {
        let us = pos.side_to_move;
        let them = us.opposite();

        // Get our king bitboard
        let king_bb = pos.pieces.get(Piece::from_parts(us, Some(PieceKind::King)));
        if king_bb.is_empty() {
            return false;
        }

        let king_sq = king_bb.0.trailing_zeros() as usize;
        let occ = pos.occupancy[OccupancyKind::Both as usize]; // could enum-wrap this too
        let king_color_mask = BitBoardMask(SQUARE_COLOR_MASK[king_sq]);

        // 1. Knights (opposite color only)
        let knight_like = pos
            .pieces
            .get(Piece::from_parts(them, Some(PieceKind::Knight)))
            & !king_color_mask;
        if !knight_like.is_empty()
            && !(BitBoardMask(KNIGHT_ATTACKS[king_sq]) & knight_like).is_empty()
        {
            return true;
        }

        // 2. Pawns (same color only)
        let pawn_like = pos
            .pieces
            .get(Piece::from_parts(them, Some(PieceKind::Pawn)))
            & king_color_mask;
        if !pawn_like.is_empty() && (PAWN_ATTACKS[them as usize][king_sq] & pawn_like.0) != 0 {
            return true;
        }

        // 3. Opponent king
        let opp_king = pos
            .pieces
            .get(Piece::from_parts(them, Some(PieceKind::King)));
        if !opp_king.is_empty() && (KING_ATTACKS[king_sq] & opp_king.0) != 0 {
            return true;
        }

        // 4. Rook/Queen (same rank/file only)
        let rook_like = (pos
            .pieces
            .get(Piece::from_parts(them, Some(PieceKind::Rook)))
            | pos
                .pieces
                .get(Piece::from_parts(them, Some(PieceKind::Queen))))
            & BitBoardMask(RANK_MASKS[king_sq] | FILE_MASKS[king_sq]);
        if !rook_like.is_empty() {
            let rmask = ROOK_MASKS[king_sq];
            let rindex = occ_to_index(occ & rmask, rmask);
            if (ROOK_ATTACKS[king_sq][rindex] & rook_like.0) != 0 {
                return true;
            }
        }

        // 5. Bishop/Queen (same color + same diagonal/antidiagonal)
        let bishop_like = (pos
            .pieces
            .get(Piece::from_parts(them, Some(PieceKind::Bishop)))
            | pos
                .pieces
                .get(Piece::from_parts(them, Some(PieceKind::Queen))))
            & king_color_mask
            & BitBoardMask(DIAGONAL_MASKS[king_sq] | ANTIDIAGONAL_MASKS[king_sq]);
        if !bishop_like.is_empty() {
            let bmask = BISHOP_MASKS[king_sq];
            let bindex = occ_to_index(occ & bmask, bmask);
            if (BISHOP_ATTACKS[king_sq][bindex] & bishop_like.0) != 0 {
                return true;
            }
        }

        false
    }
}

impl SimpleMoveGen {
    fn generate_all_pawn_moves(&self, pos: &Position, ctx: &MoveGenContext, moves: &mut Vec<Move>) {
        let pawns = pos
            .pieces
            .get(Piece::from_parts(ctx.us, Some(PieceKind::Pawn)));
        if pawns.is_empty() {
            return;
        }

        let empty = !ctx.occ;
        let their_pieces = pos.their_pieces(ctx.us); // BitBoardMask

        if ctx.us == Color::White {
            let single_push = (pawns << NORTH) & empty;
            for to in single_push.squares() {
                moves.push(Move::new((to as i8 - NORTH) as u8, to));
            }

            let double_push = ((single_push << NORTH) & empty) & BitBoardMask(RANK_4_MASK);
            for to in double_push.squares() {
                moves.push(Move::new((to as i8 - DOUBLE_NORTH) as u8, to));
            }

            let left_caps = (pawns << NORTH_WEST) & their_pieces & !BitBoardMask(FILE_H);
            for to in left_caps.squares() {
                moves.push(Move::new((to as i8 - NORTH_WEST) as u8, to));
            }

            let right_caps = (pawns << NORTH_EAST) & their_pieces & !BitBoardMask(FILE_A);
            for to in right_caps.squares() {
                moves.push(Move::new((to as i8 - NORTH_EAST) as u8, to));
            }
        } else {
            let single_push = (pawns >> -SOUTH) & empty;
            for to in single_push.squares() {
                moves.push(Move::new((to as i8 - SOUTH) as u8, to));
            }

            let double_push = ((single_push >> -SOUTH) & empty) & BitBoardMask(RANK_5_MASK);
            for to in double_push.squares() {
                moves.push(Move::new((to as i8 - DOUBLE_SOUTH) as u8, to));
            }

            let left_caps = (pawns >> -SOUTH_EAST) & their_pieces & !BitBoardMask(FILE_H);
            for to in left_caps.squares() {
                moves.push(Move::new((to as i8 - SOUTH_EAST) as u8, to));
            }

            let right_caps = (pawns >> -SOUTH_WEST) & their_pieces & !BitBoardMask(FILE_A);
            for to in right_caps.squares() {
                moves.push(Move::new((to as i8 - SOUTH_WEST) as u8, to));
            }
        }

        // TODO: en passant and promotions
    }

    fn generate_all_knight_moves(
        &self,
        pos: &Position,
        ctx: &MoveGenContext,
        moves: &mut Vec<Move>,
    ) {
        // Get all knights for the side to move
        let knights = pos
            .pieces
            .get(Piece::from_parts(ctx.us, Some(PieceKind::Knight)));
        if knights.is_empty() {
            return; // early bail
        }

        for from in knights.squares() {
            // Precomputed knight moves, masked to exclude our own pieces
            let attacks = knight_attacks_mask(from) & ctx.not_ours;
            if attacks.is_empty() {
                continue; // cheap zero-check
            }

            for to in attacks.squares() {
                moves.push(Move::new(from, to));
            }

            // Debug: knights never land on same color as origin
            debug_assert!(
                (knight_attacks_mask(from) & BitBoardMask(SQUARE_COLOR_MASK[from as usize]))
                    .is_empty(),
                "Knight attack set contains illegal squares"
            );
        }
    }

    fn generate_all_bishop_moves(
        &self,
        pos: &Position,
        ctx: &MoveGenContext,
        moves: &mut Vec<Move>,
    ) {
        // Get all bishops for the side to move
        let bishops = pos
            .pieces
            .get(Piece::from_parts(ctx.us, Some(PieceKind::Bishop)));
        if bishops.is_empty() {
            return; // early bail
        }

        for from in bishops.squares() {
            // Prefilter: same color complex + diagonals
            let reachable = ctx.not_ours
                & BitBoardMask(SQUARE_COLOR_MASK[from as usize])
                & BitBoardMask(DIAGONAL_MASKS[from as usize] | ANTIDIAGONAL_MASKS[from as usize]);
            if reachable.is_empty() {
                continue; // cheap skip
            }

            // Lookup bishop attacks from precomputed table
            let attacks = bishop_attacks_mask(from, ctx.occ) & ctx.not_ours;
            if attacks.is_empty() {
                continue; // cheap zero-check
            }

            for to in attacks.squares() {
                moves.push(Move::new(from, to));
            }

            // Debug: bishop attacks never cross color complexes
            debug_assert_eq!(
                (attacks & !BitBoardMask(SQUARE_COLOR_MASK[from as usize])).0,
                0
            );
        }
    }

    fn generate_all_rook_moves(&self, pos: &Position, ctx: &MoveGenContext, moves: &mut Vec<Move>) {
        // Get all rooks for the side to move
        let rooks = pos
            .pieces
            .get(Piece::from_parts(ctx.us, Some(PieceKind::Rook)));
        if rooks.is_empty() {
            return; // early bail
        }

        for from in rooks.squares() {
            // Prefilter: only rank/file from 'from'
            let reachable =
                ctx.not_ours & BitBoardMask(RANK_MASKS[from as usize] | FILE_MASKS[from as usize]);
            if reachable.is_empty() {
                continue;
            }

            // Lookup rook attacks from precomputed table
            let attacks = rook_attacks_mask(from, ctx.occ) & ctx.not_ours;
            if attacks.is_empty() {
                continue;
            }

            for to in attacks.squares() {
                moves.push(Move::new(from, to));
            }

            // Debug: rook attacks should be confined to same rank/file
            debug_assert_eq!(
                (attacks & !BitBoardMask(RANK_MASKS[from as usize] | FILE_MASKS[from as usize])).0,
                0
            );
        }
    }

    fn generate_all_queen_moves(
        &self,
        pos: &Position,
        ctx: &MoveGenContext,
        moves: &mut Vec<Move>,
    ) {
        let queens = pos
            .pieces
            .get(Piece::from_parts(ctx.us, Some(PieceKind::Queen)));
        if queens.is_empty() {
            return; // early bail
        }

        for from in queens.squares() {
            // Prefilter: queens can only move along rank/file/diagonals
            let reachable = ctx.not_ours
                & BitBoardMask(
                    RANK_MASKS[from as usize]
                        | FILE_MASKS[from as usize]
                        | DIAGONAL_MASKS[from as usize]
                        | ANTIDIAGONAL_MASKS[from as usize],
                );
            if reachable.is_empty() {
                continue; // no possible moves along queen lines
            }

            // Rook-like component
            let rook_attacks = rook_attacks_mask(from, ctx.occ);

            // Bishop-like component
            let bishop_attacks = bishop_attacks_mask(from, ctx.occ);

            // Combine and mask with not_ours
            let attacks = (rook_attacks | bishop_attacks) & ctx.not_ours;
            if attacks.is_empty() {
                continue; // cheap zero-check
            }

            for to in attacks.squares() {
                moves.push(Move::new(from, to));
            }

            // Debug geometry validation
            debug_assert_eq!(
                (rook_attacks
                    & !BitBoardMask(RANK_MASKS[from as usize] | FILE_MASKS[from as usize]))
                .0,
                0
            );
            debug_assert_eq!(
                (bishop_attacks
                    & !BitBoardMask(
                        DIAGONAL_MASKS[from as usize] | ANTIDIAGONAL_MASKS[from as usize]
                    ))
                .0,
                0
            );
        }
    }

    fn generate_all_king_moves(&self, pos: &Position, ctx: &MoveGenContext, moves: &mut Vec<Move>) {
        // Get the king bitboard for the side to move
        let king_bb = pos
            .pieces
            .get(Piece::from_parts(ctx.us, Some(PieceKind::King)));
        if king_bb.is_empty() {
            return; // no king found — should never happen in a legal position
        }

        for from in king_bb.squares() {
            // Precomputed king moves, masked to exclude our own pieces
            let attacks = king_attacks_mask(from) & ctx.not_ours;
            if attacks.is_empty() {
                continue; // cheap zero-check
            }

            for to in attacks.squares() {
                moves.push(Move::new(from, to));
            }

            // Debug: king attacks must be within one square in any direction
            debug_assert!(
                (attacks & !king_attacks_mask(from)).is_empty(),
                "King attack set contains illegal squares"
            );
        }

        // TODO: Castling logic can be added here if desired
    }
}

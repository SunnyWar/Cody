// src/search/movegen.rs

use crate::core::bitboard::{
    ANTIDIAGONAL_MASKS, BISHOP_ATTACKS, BISHOP_MASKS, DIAGONAL_MASKS, KING_ATTACKS, KNIGHT_ATTACKS,
    PAWN_ATTACKS, ROOK_ATTACKS, ROOK_MASKS, bishop_attacks_from, gen_king_attacks,
    knight_attacks_for, occupancy_to_index, rook_attacks_from,
};
use crate::core::bitboardmask::BitBoardMask;
use crate::core::mov::Move;
use crate::core::occupancy::OccupancyKind;
use crate::core::piece::{Color, Piece, PieceKind};
use crate::core::position::{MoveGenContext, Position};
use crate::generated::{FILE_A, FILE_H, FILE_MASKS, RANK_MASKS, SQUARE_COLOR_MASK};

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
    fn in_check(&self, pos: &Position) -> bool;
}

impl MoveGenerator for SimpleMoveGen {
    fn in_check(&self, pos: &Position) -> bool {
        let us = pos.side_to_move;
        let them = us.opposite();

        let king_bb = pos.pieces.get(Piece::from_parts(us, Some(PieceKind::King)));
        if king_bb.is_empty() {
            return false;
        }

        let king_sq = king_bb.0.trailing_zeros() as usize;
        let occupancy = pos.occupancyupancy[OccupancyKind::Both];
        let king_color_mask = BitBoardMask(SQUARE_COLOR_MASK[king_sq]);

        // 1. Knights (opposite color only)
        let knight_like = pos
            .pieces
            .get(Piece::from_parts(them, Some(PieceKind::Knight)))
            & !king_color_mask;
        if !knight_like.is_empty() && (KNIGHT_ATTACKS[king_sq] & knight_like).is_nonempty() {
            return true;
        }

        // 2. Pawns (same color only)
        let pawn_like = pos
            .pieces
            .get(Piece::from_parts(them, Some(PieceKind::Pawn)))
            & king_color_mask;
        if !pawn_like.is_empty() && (PAWN_ATTACKS[them as usize][king_sq] & pawn_like).is_nonempty()
        {
            return true;
        }

        // 3. Opponent king
        let opp_king = pos
            .pieces
            .get(Piece::from_parts(them, Some(PieceKind::King)));
        if !opp_king.is_empty() && (KING_ATTACKS[king_sq] & opp_king).is_nonempty() {
            return true;
        }

        // 4. Rook/Queen (same rank/file only)
        let rook_like = (pos
            .pieces
            .get(Piece::from_parts(them, Some(PieceKind::Rook)))
            | pos
                .pieces
                .get(Piece::from_parts(them, Some(PieceKind::Queen))))
            & RANK_MASKS[king_sq]
            | FILE_MASKS[king_sq];
        if !rook_like.is_empty() {
            let rmask = ROOK_MASKS[king_sq];
            let rindex = occupancy_to_index(occupancy, rmask);
            if (ROOK_ATTACKS[king_sq][rindex] & rook_like).is_nonempty() {
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
            & DIAGONAL_MASKS[king_sq]
            | ANTIDIAGONAL_MASKS[king_sq];

        if !bishop_like.is_empty() {
            let bmask = BISHOP_MASKS[king_sq];
            let bindex = occupancy_to_index(occupancy, bmask);
            if (BISHOP_ATTACKS[king_sq][bindex] & bishop_like).is_nonempty() {
                return true;
            }
        }

        false
    }
}

pub fn generate_moves(pos: &Position) -> Vec<Move> {
    let mut moves = Vec::new();
    let context = MoveGenContext {
        us: pos.side_to_move,
        occupancy: pos.all_pieces(),
        not_ours: !pos.our_pieces(pos.side_to_move),
    };

    generate_all_pawn_moves(pos, &context, &mut moves);
    generate_all_knight_moves(pos, &context, &mut moves);
    generate_all_bishop_moves(pos, &context, &mut moves);
    generate_all_rook_moves(pos, &context, &mut moves);
    generate_all_queen_moves(pos, &context, &mut moves);
    generate_all_king_moves(pos, &context, &mut moves);

    moves
}

fn generate_all_pawn_moves(pos: &Position, context: &MoveGenContext, moves: &mut Vec<Move>) {
    let pawns = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Pawn)));
    if pawns.is_empty() {
        return;
    }

    let empty = !context.occupancy;
    let their_pieces = pos.their_pieces(context.us);

    let (single_push_dir, double_push_dir, left_cap_dir, right_cap_dir, double_rank_mask) =
        match context.us {
            Color::White => (
                NORTH,
                DOUBLE_NORTH,
                NORTH_WEST,
                NORTH_EAST,
                BitBoardMask(RANK_4_MASK),
            ),
            Color::Black => (
                SOUTH,
                DOUBLE_SOUTH,
                SOUTH_EAST,
                SOUTH_WEST,
                BitBoardMask(RANK_5_MASK),
            ),
        };

    // Single push
    let single_push = (pawns << single_push_dir) & empty;
    for to in single_push.squares() {
        if let Some(from) = to.advance(-single_push_dir) {
            moves.push(Move::new(from, to));
        }
    }

    // Double push
    let double_push = ((single_push << single_push_dir) & empty) & double_rank_mask;
    for to in double_push.squares() {
        if let Some(from) = to.advance(-double_push_dir) {
            moves.push(Move::new(from, to));
        }
    }

    // Left capture
    let left_caps = (pawns << left_cap_dir) & their_pieces & !FILE_H;
    for to in left_caps.squares() {
        if let Some(from) = to.advance(-left_cap_dir) {
            moves.push(Move::new(from, to));
        }
    }

    // Right capture
    let right_caps = (pawns << right_cap_dir) & their_pieces & !FILE_A;
    for to in right_caps.squares() {
        if let Some(from) = to.advance(-right_cap_dir) {
            moves.push(Move::new(from, to));
        }
    }

    // TODO: en passant and promotions
}

// TODO - this can probably be improved by have an attack mask
// TODO - generated by a 5x5 move mask moved around to every square added
// TODO - making final 8x8 can be masked with current knight location to make
// TODO - new mask that only shows possible moves then mask that will
// TODO - opponent locations and empty square to show all possible moves
// TODO - all with masking!
fn generate_all_knight_moves(pos: &Position, context: &MoveGenContext, moves: &mut Vec<Move>) {
    // Get a bitboard of all knights for the current side.
    let knights = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Knight)));

    // Iterate over each square containing one of our knights.
    for from in knights.squares() {
        // Calculate all squares this knight can move to, filtered by squares
        // not occupancyupied by our own pieces.
        let valid_moves = knight_attacks_for(from).and(context.not_ours);

        // For each valid destination square, create and record the move.
        for to in valid_moves.squares() {
            moves.push(Move::new(from, to));
        }
    }
}

fn generate_all_bishop_moves(pos: &Position, context: &MoveGenContext, moves: &mut Vec<Move>) {
    // Get a bitboard of all bishops for the current side.
    let bishops = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Bishop)));

    // Iterate over each square containing one of our bishops. This is a great, type-safe pattern.
    for from in bishops.squares() {
        // 1. Calculate all squares this bishop attacks, using the board's
        //    total occupancyupancy (`context.occupancy`) to identify blockers.
        let attacks = bishop_attacks_from(from, context.occupancy);

        // 2. Filter these attacks to find valid moves. A valid move ends on a square
        //    that is NOT occupancyupied by one of our own pieces (`context.not_ours`).
        let valid_moves = attacks.and(context.not_ours);

        // 3. For each valid destination square, create and record the move.
        for to in valid_moves.squares() {
            moves.push(Move::new(from, to));
        }
    }
}

fn generate_all_rook_moves(pos: &Position, context: &MoveGenContext, moves: &mut Vec<Move>) {
    // Get a bitboard of all rooks for the current side.
    let rooks = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Rook)));

    // Iterate over each square containing one of our rooks.
    for from in rooks.squares() {
        // Calculate all squares this rook attacks, using the board's total occupancyupancy.
        let valid_moves = rook_attacks_from(from, context.occupancy).and(context.not_ours);

        // For each valid destination square, create and record the move.
        for to in valid_moves.squares() {
            moves.push(Move::new(from, to));
        }
    }
}

fn generate_all_queen_moves(pos: &Position, context: &MoveGenContext, moves: &mut Vec<Move>) {
    // Get a bitboard of all queens for the current side.
    let queens = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::Queen)));

    // Iterate over each square containing one of our queens.
    for from in queens.squares() {
        // Calculate all squares this queen can move to (rook-like and bishop-like moves).
        let valid_moves = (rook_attacks_from(from, context.occupancy)
            | bishop_attacks_from(from, context.occupancy))
        .and(context.not_ours);

        // For each valid destination square, create and record the move.
        for to in valid_moves.squares() {
            moves.push(Move::new(from, to));
        }
    }
}

fn generate_all_king_moves(pos: &Position, context: &MoveGenContext, moves: &mut Vec<Move>) {
    // Get the bitboard of the king for the current side (exactly one square in standard chess).
    let king_bb = pos
        .pieces
        .get(Piece::from_parts(context.us, Some(PieceKind::King)));

    // Get the single square where the king is located.
    if let Some(from) = king_bb.squares().next() {
        // Calculate all squares this king can move to, filtered by squares
        // not occupancyupied by our own pieces.
        let valid_moves = gen_king_attacks(from).and(context.not_ours);

        // For each valid destination square, create and record the move.
        for to in valid_moves.squares() {
            moves.push(Move::new(from, to));
        }
    }

    // TODO: Castling logic can be added here if desired
}

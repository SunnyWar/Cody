#[cfg(test)]
mod knight_movegen_unit_tests {
    use crate::position::MoveGenContext;
    use crate::position::Position;

    fn collect_knight_moves(fen: &str) -> Vec<(String, String)> {
        let pos = Position::from_fen(fen);
        let mut moves = Vec::new();
        let context = MoveGenContext {
            us: pos.side_to_move,
            occupancy: pos.all_pieces(),
            not_ours: !pos.our_pieces(pos.side_to_move),
        };
        super::generate_pseudo_knight_moves(&pos, &context, &mut moves);
        moves
            .iter()
            .map(|m| (m.from().to_string(), m.to().to_string()))
            .collect()
    }

    #[test]
    fn knight_moves_from_center() {
        let moves = collect_knight_moves("8/8/8/8/3N4/8/8/8 w - - 0 1");
        let found: Vec<_> = moves
            .iter()
            .filter(|(from, _)| from == "d4")
            .map(|(_, to)| to.clone())
            .collect();
        assert_eq!(found.len(), 8, "Knight should have 8 moves from d4");
    }

    #[test]
    fn knight_moves_from_corner() {
        let moves = collect_knight_moves("N7/8/8/8/8/8/8/8 w - - 0 1");
        let found: Vec<_> = moves
            .iter()
            .filter(|(from, _)| from == "a1")
            .map(|(_, to)| to.clone())
            .collect();
        let expected = ["b3", "c2"];
        for sq in &expected {
            assert!(
                found.contains(&sq.to_string()),
                "Missing knight move to {}",
                sq
            );
        }
        assert_eq!(found.len(), 2, "Knight should have 2 moves from a1");
    }

    #[test]
    fn knight_moves_from_edge() {
        let moves = collect_knight_moves("8/8/8/8/8/8/8/7N w - - 0 1");
        let found: Vec<_> = moves
            .iter()
            .filter(|(from, _)| from == "h1")
            .map(|(_, to)| to.clone())
            .collect();
        let expected = ["f2", "g3"];
        for sq in &expected {
            assert!(
                found.contains(&sq.to_string()),
                "Missing knight move to {}",
                sq
            );
        }
        assert_eq!(found.len(), 2, "Knight should have 2 moves from h1");
    }

    #[test]
    fn knight_moves_blocked_by_own_piece() {
        let moves = collect_knight_moves("8/8/8/8/3N4/8/4P3/8 w - - 0 1");
        let found: Vec<_> = moves
            .iter()
            .filter(|(from, _)| from == "d4")
            .map(|(_, to)| to.clone())
            .collect();
        assert!(
            !found.contains(&"e6".to_string()),
            "Knight should not move to e6 (own piece)"
        );
    }

    #[test]
    fn knight_moves_can_capture_opponent() {
        let moves = collect_knight_moves("8/8/8/8/3N4/8/4p3/8 w - - 0 1");
        let found: Vec<_> = moves
            .iter()
            .filter(|(from, _)| from == "d4")
            .map(|(_, to)| to.clone())
            .collect();
        assert!(
            found.contains(&"e6".to_string()),
            "Knight should be able to capture on e6"
        );
    }

    #[test]
    fn knight_moves_from_h8() {
        let moves = collect_knight_moves("7N/8/8/8/8/8/8/8 w - - 0 1");
        let found: Vec<_> = moves
            .iter()
            .filter(|(from, _)| from == "h8")
            .map(|(_, to)| to.clone())
            .collect();
        let expected = ["f7", "g6"];
        for sq in &expected {
            assert!(
                found.contains(&sq.to_string()),
                "Missing knight move to {}",
                sq
            );
        }
        assert_eq!(found.len(), 2, "Knight should have 2 moves from h8");
    }
}

#[cfg(test)]
mod movegen_regression_tests {
    use super::*;
    use crate::bitboard::square::Square;
    use crate::position::Position;

    #[test]
    fn test_knight_moves_high_rank() {
        let pos = Position::from_fen("7N/8/8/8/8/8/8/8 w - - 0 1");
        let knight_bb = pos.pieces.get(crate::piece::Piece::from_parts(
            crate::piece::Color::White,
            Some(crate::piece::PieceKind::Knight),
        ));
        println!("Knight bitboard: 0x{:016x}", knight_bb.0);
        let moves = generate_legal_moves(&pos);
        println!("All generated moves for knight on h8:");
        for m in &moves {
            println!(
                "{:?} {} -> {}",
                m.move_type,
                m.from().to_string(),
                m.to().to_string()
            );
        }
        let found: Vec<_> = moves
            .iter()
            .filter(|m| {
                m.from().to_string() == "g8"
                    && (m.to().to_string() == "f6" || m.to().to_string() == "g6")
            })
            .collect();
        assert_eq!(found.len(), 2, "Knight on h8 should have 2 moves");
        assert!(
            found.iter().any(|m| m.to().to_string() == "g6"),
            "Missing knight move to g6"
        );
        assert!(
            found.iter().any(|m| m.to().to_string() == "f6"),
            "Missing knight move to f6"
        );
    }

    #[test]
    fn test_rook_moves_high_rank() {
        let pos = Position::from_fen("7R/8/8/8/8/8/8/8 w - - 0 1");
        let moves = generate_legal_moves(&pos);
        println!("All generated moves for rook on h8:");
        for m in &moves {
            println!("{:?} {} -> {}", m.move_type, m.from(), m.to());
        }
        let found: Vec<_> = moves
            .iter()
            .filter(|m| m.from().to_string() == "h8")
            .map(|m| m.to().to_string())
            .collect();
        assert_eq!(found.len(), 14, "Rook on h8 should have 14 moves");
        assert!(found.contains(&"h1".to_string()), "Missing rook move to h1");
        assert!(found.contains(&"a8".to_string()), "Missing rook move to a8");
    }

    #[test]
    fn test_pawn_promotion_high_rank() {
        let pos = Position::from_fen("8/7P/8/8/8/8/8/8 w - - 0 1");
        let moves = generate_legal_moves(&pos);
        println!("All generated moves for pawn on h7:");
        for m in &moves {
            println!("{:?} {} -> {}", m.move_type, m.from(), m.to());
        }
        let found: Vec<_> = moves
            .iter()
            .filter(|m| m.from().to_string() == "h7" && m.to().to_string() == "h8")
            .collect();
        assert!(
            !found.is_empty(),
            "Pawn on h7 should be able to promote to h8"
        );
    }

    #[test]
    fn test_en_passant_high_rank() {
        let pos = Position::from_fen("8/8/8/6Pp/8/8/8/8 w - h6 0 1");
        let moves = generate_legal_moves(&pos);
        println!("All generated moves for pawn on g5 (en passant test):");
        for m in &moves {
            println!(
                "{:?} {} -> {} | from idx {} to idx {} | move_type: {:?} | from_str: {:?} to_str: ",
                m.move_type,
                m.from(),
                m.to(),
                m.from().index(),
                m.to().index(),
                m.move_type,
                m.from(),
                m.to()
            );
        }
        let found: Vec<_> = moves
            .iter()
            .filter(|m| {
                m.from().to_string() == "g5"
                    && m.to().to_string() == "h6"
                    && matches!(m.move_type, MoveType::EnPassant)
            })
            .collect();
        println!("Found vector length: {}", found.len());
        println!("Found moves:");
        for m in &found {
            println!(
                "{:?} {} -> {}",
                m.move_type,
                m.from().to_string(),
                m.to().to_string()
            );
        }
        assert!(
            !found.is_empty(),
            "En passant capture from g5 to h6 should be available"
        );
    }

    #[test]
    fn test_bishop_moves_blocked_by_own_piece() {
        let pos = Position::from_fen("8/8/8/8/8/8/2P5/2B5 w - - 0 1");
        let moves = generate_legal_moves(&pos);
        for m in &moves {
            assert_ne!(
                m.to(),
                Square::from_coords('d', '2').unwrap(),
                "Bishop should not capture own pawn"
            );
        }
    }
}

/// Integration test demonstrating perft usage
/// Run with: cargo test --lib test_perft_integration -- --nocapture
#[cfg(test)]
mod perft_integration_tests {
    use bitboard::Square;
    use bitboard::mov::ChessMove;
    use bitboard::mov::MoveType;
    use bitboard::movegen::generate_legal_moves;
    use bitboard::occupancy::OccupancyKind;
    use bitboard::perft;
    use bitboard::piece::Color;
    use bitboard::piece::Piece;
    use bitboard::piece::PieceKind;
    use bitboard::position::Position;

    fn moving_piece_kind(pos: &Position, mv: &ChessMove) -> PieceKind {
        for kind in [
            PieceKind::Pawn,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Rook,
            PieceKind::Queen,
            PieceKind::King,
        ] {
            let piece = Piece::from_parts(pos.side_to_move, Some(kind));
            if pos.pieces.get(piece).contains(mv.from()) {
                return kind;
            }
        }

        panic!("No piece found on {}", mv.from());
    }

    fn is_capture_move(pos: &Position, mv: &ChessMove) -> bool {
        match mv.move_type {
            MoveType::Capture | MoveType::EnPassant => true,
            MoveType::Promotion(_) => {
                let them = pos.side_to_move.opposite();
                let occ = match them {
                    Color::White => OccupancyKind::White,
                    Color::Black => OccupancyKind::Black,
                };
                (pos.occupancy[occ] & mv.to().bitboard()).is_nonempty()
            }
            _ => false,
        }
    }

    fn resolve_san_like_move(pos: &Position, token: &str) -> ChessMove {
        let mut move_str = token.trim().to_ascii_lowercase();
        while matches!(move_str.chars().last(), Some('+') | Some('#')) {
            move_str.pop();
        }

        let mut promotion = None;
        if move_str.len() >= 3 {
            let bytes = move_str.as_bytes();
            let last = bytes[bytes.len() - 1] as char;
            let rank = bytes[bytes.len() - 2] as char;
            let file = bytes[bytes.len() - 3] as char;
            if "qrbn".contains(last) && ('1'..='8').contains(&rank) && ('a'..='h').contains(&file) {
                promotion = PieceKind::from_uci(last);
                move_str.pop();
            }
        }

        let len = move_str.len();
        assert!(len >= 2, "Invalid move token: {token}");

        let dest_file = move_str.as_bytes()[len - 2] as char;
        let dest_rank = move_str.as_bytes()[len - 1] as char;
        let dest_sq = Square::from_coords(dest_file, dest_rank)
            .unwrap_or_else(|| panic!("Invalid destination in {token}"));

        let is_capture = move_str.contains('x');

        let first = move_str.as_bytes()[0] as char;
        let (piece_kind, disambig) = if len == 2 {
            (PieceKind::Pawn, "")
        } else {
            match first {
                'n' => (PieceKind::Knight, &move_str[1..len - 2]),
                'b' => (PieceKind::Bishop, &move_str[1..len - 2]),
                'r' => (PieceKind::Rook, &move_str[1..len - 2]),
                'q' => (PieceKind::Queen, &move_str[1..len - 2]),
                'k' => (PieceKind::King, &move_str[1..len - 2]),
                _ => (PieceKind::Pawn, &move_str[..len - 2]),
            }
        };

        let mut dis_file = None;
        let mut dis_rank = None;
        if piece_kind != PieceKind::Pawn {
            let mut cleaned = disambig.to_string();
            cleaned.retain(|c| c != 'x');
            for ch in cleaned.chars() {
                if ('a'..='h').contains(&ch) {
                    dis_file = Some(ch);
                } else if ('1'..='8').contains(&ch) {
                    dis_rank = Some(ch);
                }
            }
        }

        let mut candidates: Vec<ChessMove> = Vec::new();
        for mv in generate_legal_moves(pos) {
            if mv.to() != dest_sq {
                continue;
            }
            if mv.promotion() != promotion {
                continue;
            }

            let moving_kind = moving_piece_kind(pos, &mv);
            if moving_kind != piece_kind {
                continue;
            }

            if is_capture != is_capture_move(pos, &mv) {
                continue;
            }

            if moving_kind == PieceKind::Pawn && is_capture {
                let origin_file = move_str.as_bytes()[0] as char;
                if mv.from().file_char() != origin_file {
                    continue;
                }
            }

            if let Some(file) = dis_file {
                if mv.from().file_char() != file {
                    continue;
                }
            }
            if let Some(rank) = dis_rank {
                if mv.from().rank_char() != rank {
                    continue;
                }
            }

            candidates.push(mv);
        }

        assert_eq!(
            candidates.len(),
            1,
            "Ambiguous or missing move for {token}: {candidates:?}"
        );

        candidates[0]
    }

    #[test]
    fn test_perft_initial_position_depth_2() {
        let pos = Position::default();
        let count = perft(&pos, 2);
        println!("Initial position: perft(2) = {}", count);
        assert!(count > 0, "Should have legal moves");
    }

    #[test]
    fn test_perft_divide_example() {
        let pos = Position::default();
        println!("\nInitial position move breakdown (depth 1):");

        // At depth 1, each move represents itself
        let moves = bitboard::movegen::generate_legal_moves(&pos);
        println!("Total moves: {}", moves.len());

        for (i, mv) in moves.iter().take(5).enumerate() {
            println!("  Move {}: {} -> {}", i + 1, mv.from, mv.to);
        }
    }

    #[test]
    fn test_perft_kiwipete_position() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 10";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "a2a3", "b2b3", "g2g3", "d5d6", "a2a4", "g2g4", "g2h3", "d5e6", "c3b1", "c3d1", "c3a4",
            "c3b5", "e5d3", "e5c4", "e5g4", "e5c6", "e5g6", "e5d7", "e5f7", "d2c1", "d2e3", "d2f4",
            "d2g5", "d2h6", "e2d1", "e2f1", "e2d3", "e2c4", "e2b5", "e2a6", "a1b1", "a1c1", "a1d1",
            "h1f1", "h1g1", "f3d3", "f3e3", "f3g3", "f3h3", "f3f4", "f3g4", "f3f5", "f3h5", "f3f6",
            "e1d1", "e1f1", "e1g1", "e1c1",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in Kiwipete position"
        );
    }

    #[test]
    fn test_perft_mirrored_stress_position() {
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec!["b4c5", "c4c5", "d2d4", "f1f2", "f3d4", "g1h1"];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in mirrored stress position"
        );
    }

    #[test]
    fn test_perft_promotion_pin_stress_position() {
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        let pos = Position::from_fen(fen);

        let moves = generate_legal_moves(&pos);
        assert_eq!(moves.len(), 44, "Unexpected legal move count");

        let move_strs: Vec<String> = moves.iter().map(|m| m.to_string()).collect();
        for promo in ["d7c8q", "d7c8r", "d7c8b", "d7c8n"] {
            assert!(
                move_strs.contains(&promo.to_string()),
                "Missing move {promo}"
            );
        }
    }

    #[test]
    fn test_perft_endgame_position() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 11";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "e2e3", "g2g3", "a5a6", "e2e4", "g2g4", "b4b1", "b4b2", "b4b3", "b4a4", "b4c4", "b4d4",
            "b4e4", "b4f4", "a5a4",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in endgame position"
        );
    }

    #[test]
    fn test_perft_midgame_position() {
        let fen = "r1b1r1k1/1p1nqppp/p1pb1n2/3p4/2PPp2P/1PN1PNP1/P1Q1BP2/R1BR2K1 b - - 0 3";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "a6a5", "c6c5", "b7b6", "g7g6", "h7h6", "b7b5", "g7g5", "h7h5", "d5c4", "e4f3", "f6g4",
            "f6h5", "d7c5", "d7e5", "d7b6", "d7b8", "d7f8", "d6a3", "d6g3", "d6b4", "d6f4", "d6c5",
            "d6e5", "d6c7", "d6b8", "a8a7", "a8b8", "e8d8", "e8f8", "e7e5", "e7e6", "e7d8", "e7f8",
            "g8f8", "g8h8",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in midgame position"
        );
    }

    #[test]
    fn test_perft_midgame_position_white_illegal_promotion() {
        let fen = "rn1qnrk1/1P3p2/6pp/1p4bP/PpB1p1P1/1P2PN2/2Q2P2/R1B2RK1 w - - 0 12";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "a4a5", "b7a8q", "b7a8r", "b7a8b", "b7a8n", "a4b5", "h5g6", "f3e1", "f3d2", "f3h2",
            "f3d4", "f3h4", "f3e5", "f3g5", "c1b2", "c1d2", "c1a3", "c4e2", "c4d3", "c4b5", "c4d5",
            "c4e6", "c4f7", "a1b1", "a1a2", "a1a3", "f1d1", "f1e1", "c2b1", "c2d1", "c2a2", "c2b2",
            "c2d2", "c2e2", "c2c3", "c2d3", "c2e4", "g1h1", "g1g2", "g1h2",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in midgame promotion position"
        );
    }

    #[test]
    fn test_perft_limited_legal_moves_position() {
        let fen = "3qk2r/3P3p/rnn1p2P/2b3p1/2B2NP1/4pP2/1R6/1qB1K2b b k - 0 20";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec!["b6d7", "d8d7", "e8d7", "e8e7", "e8f7", "e8f8"];
        expected.sort();

        assert_eq!(moves, expected, "Unexpected legal moves in position");
    }

    #[test]
    fn test_perft_two_queens_illegal_move() {
        let fen = "r1bq1rk1/1p4pp/p7/N2p1p2/P2P1P2/bP2B1PP/4P3/R2q1RKB w - - 0 9";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "b3b4", "g3g4", "h3h4", "a5c4", "a5c6", "a5b7", "h1g2", "h1f3", "h1e4", "h1d5", "e3c1",
            "e3d2", "e3f2", "a1b1", "a1c1", "a1d1", "a1a2", "a1a3", "f1d1", "f1e1", "g1h2", "g1g2",
            "g1f2",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in two-queens position"
        );
    }

    #[test]
    fn test_promotion_move_consistency() {
        // FEN just before Black plays c2xd1=Q (capture pawn on c2 takes rook on d1 and
        // promotes to queen)
        let fen = "r1bq1rk1/pp2bppp/2n5/3p4/P2P4/4B1P1/1PpNPP1P/R2Q1RKB b - - 0 3";
        let pos = Position::from_fen(fen);
        let original_hash = pos.zobrist_hash();

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "a7a6", "b7b6", "f7f6", "g7g6", "h7h6", "a7a5", "b7b5", "f7f5", "g7g5", "h7h5",
            "c2d1q", "c2d1r", "c2d1b", "c2d1n", "c2c1q", "c2c1r", "c2c1b", "c2c1n", "c6b4", "c6d4",
            "c6a5", "c6e5", "c6b8", "e7a3", "e7b4", "e7h4", "e7c5", "e7g5", "e7d6", "e7f6", "c8h3",
            "c8g4", "c8f5", "c8e6", "c8d7", "a8b8", "f8e8", "d8a5", "d8b6", "d8d6", "d8c7", "d8d7",
            "d8e8", "g8h8",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in promotion consistency position"
        );

        // Parse the promotion move (capture + promote)
        let m_str = "c2d1q";
        let m = pos
            .parse_uci_move(m_str)
            .expect("Failed to parse promotion move");
        assert_eq!(m.from(), Square::C2, "Move from should be c2");
        assert_eq!(m.to(), Square::D1, "Move to should be d1");
        assert_eq!(
            m.promotion(),
            Some(bitboard::piece::PieceKind::Queen),
            "Move should be a queen promotion"
        );

        // Execute the promotion
        let mut new_pos = Position::default();
        pos.apply_move_into(&m, &mut new_pos);

        // Verify hash changed after the move
        let new_hash = new_pos.zobrist_hash();
        assert_ne!(
            new_hash, original_hash,
            "Hash should change after making a move"
        );

        // Verify the promoted piece is on d1 and is a Black Queen
        let black_queen = Piece::from_parts(Color::Black, Some(bitboard::piece::PieceKind::Queen));
        assert!(
            new_pos.pieces.get(black_queen).contains(Square::D1),
            "Promoted Black Queen not found on d1"
        );

        // Simulate undo by reconstructing from FEN and verifying consistency
        let reconstructed_pos = Position::from_fen(&pos.to_fen());
        assert_eq!(
            reconstructed_pos.zobrist_hash(),
            original_hash,
            "Reconstructed position hash should match original"
        );
        assert_eq!(
            reconstructed_pos.to_fen(),
            pos.to_fen(),
            "Reconstructed position FEN should match original"
        );
    }

    #[test]
    fn test_en_passant_rank_pin_position() {
        let fen = "7k/8/8/K1pP3r/8/8/8/8 w - c6 0 1";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec!["a5a4", "a5a6", "a5b5", "a5b6", "d5d6"];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in en passant rank pin position"
        );
    }

    #[test]
    fn test_perft_symmetrical_midgame_position() {
        let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        let pos = Position::from_fen(fen);

        let moves = generate_legal_moves(&pos);
        assert_eq!(moves.len(), 46, "Unexpected legal move count");
    }

    #[test]
    fn test_two_move_restricted_position() {
        let fen = "r5k1/3R4/2P2b1p/1p2n2p/p3q2P/P7/8/R3q1K1 w - - 0 37";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec!["a1e1", "g1h2"];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in two-move restricted position"
        );
    }

    #[test]
    fn test_complex_rook_position() {
        let fen = "r2q1R2/3P2k1/p4R2/P3b3/5R1p/6pP/6K1/2R5 b - - 2 54";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "e5a1", "e5b2", "e5c3", "e5d4", "e5f4", "e5d6", "e5f6", "e5c7", "e5b8", "a8a7", "a8b8",
            "a8c8", "d8a5", "d8b6", "d8f6", "d8c7", "d8d7", "d8e7", "d8b8", "d8c8", "d8e8", "d8f8",
            "g7h7",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in complex rook position"
        );
    }

    #[test]
    fn test_complex_piece_position() {
        let fen = "r2q1rk1/pb2b1p1/1n3p2/1PpP2P1/2P4N/R1B5/3N1pBK/3q4 w - - 0 18";
        let pos = Position::from_fen(fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "d5d6", "g5g6", "g5f6", "d2b1", "d2f1", "d2b3", "d2f3", "d2e4", "h4f3", "h4f5", "h4g6",
            "g2f1", "g2h1", "g2f3", "g2h3", "g2e4", "c3a1", "c3b2", "c3b4", "c3d4", "c3a5", "c3e5",
            "c3f6", "a3a1", "a3a2", "a3b3", "a3a4", "a3a5", "a3a6", "a3a7", "h2h3", "h2g3",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in complex piece position"
        );
    }

    #[test]
    fn test_illegal_move_not_generated_in_complex_position() {
        // Position where d1 is a BLACK QUEEN and d1-e1 should NOT be legal for White
        let game_fen = "r1bq1rk1/1p4pp/p7/N2p1p2/P2P1P2/bP2B1PP/4P3/R2q1RKB w - - 0 9";
        let pos_with_black_queen = Position::from_fen(game_fen);

        let mut moves: Vec<String> = generate_legal_moves(&pos_with_black_queen)
            .iter()
            .map(|m| m.to_string())
            .collect();
        moves.sort();

        let mut expected = vec![
            "b3b4", "g3g4", "h3h4", "a5c4", "a5c6", "a5b7", "h1g2", "h1f3", "h1e4", "h1d5", "e3c1",
            "e3d2", "e3f2", "a1b1", "a1c1", "a1d1", "a1a2", "a1a3", "f1d1", "f1e1", "g1h2", "g1g2",
            "g1f2",
        ];
        expected.sort();

        assert_eq!(
            moves, expected,
            "Unexpected legal moves in complex position"
        );
    }

    #[test]
    fn test_game_sequence_integrity() {
        let start_fen = "r2q1rk1/p2nbppp/bpp1pn2/3p4/2PP4/1PB2NP1/P2NPPBP/R2Q1RK1 w - - 0 1";
        let mut pos = Position::from_fen(start_fen);

        let moves = [
            "g4", "c5", "e4", "h6", "b4", "e5", "a3", "h5", "h3", "h4", "nxh4", "dxe4", "a4", "e3",
            "a5", "e2", "axb6", "exf1q", "kh2", "e4", "d5", "qxd1", "nb1", "e3", "b5", "exf2",
            "ra3", "nxg4", "hxg4", "bb7", "nd2", "nxb6", "g5", "f6",
        ];

        for token in moves {
            let mv = resolve_san_like_move(&pos, token);
            let mut next_pos = Position::default();
            pos.apply_move_into(&mv, &mut next_pos);
            pos = next_pos;
        }

        let final_fen = pos.to_fen();
        let clean_pos = Position::from_fen(&final_fen);

        for color in [bitboard::piece::Color::White, bitboard::piece::Color::Black] {
            for kind in [
                bitboard::piece::PieceKind::Pawn,
                bitboard::piece::PieceKind::Knight,
                bitboard::piece::PieceKind::Bishop,
                bitboard::piece::PieceKind::Rook,
                bitboard::piece::PieceKind::Queen,
                bitboard::piece::PieceKind::King,
            ] {
                let piece = bitboard::piece::Piece::from_parts(color, Some(kind));
                assert_eq!(
                    pos.pieces.get(piece),
                    clean_pos.pieces.get(piece),
                    "Bitboard corruption detected for piece {:?}",
                    piece
                );
            }
        }
        assert_eq!(
            pos.occupancy[bitboard::occupancy::OccupancyKind::White],
            clean_pos.occupancy[bitboard::occupancy::OccupancyKind::White],
            "White occupancy mismatch after sequence"
        );
        assert_eq!(
            pos.occupancy[bitboard::occupancy::OccupancyKind::Black],
            clean_pos.occupancy[bitboard::occupancy::OccupancyKind::Black],
            "Black occupancy mismatch after sequence"
        );
        assert_eq!(
            pos.zobrist_hash(),
            clean_pos.zobrist_hash(),
            "Hash corruption: make/apply logic is desynced"
        );

        let mut played_moves: Vec<String> = generate_legal_moves(&pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        let mut clean_moves: Vec<String> = generate_legal_moves(&clean_pos)
            .iter()
            .map(|m| m.to_string())
            .collect();
        played_moves.sort();
        clean_moves.sort();

        assert_eq!(
            played_moves, clean_moves,
            "The engine found different moves via play than via FEN"
        );
    }
}

// src/api.rs

use bitboard::movegen::SimpleMoveGen;
use bitboard::piece::Color;
use bitboard::position::Position;
use engine::{Engine, MaterialEvaluator, NODE_COUNT, TEST_CASES, TestCase};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::api::golimits::GoLimits;

pub const MATE_SCORE: i32 = 30_000; // or 32_000, or 10_000

pub struct CodyApi {
    engine: Engine<SimpleMoveGen, MaterialEvaluator>,
    current_pos: Position,
    limits: GoLimits,
    stop: Arc<AtomicBool>, // for future: stop support
}

impl CodyApi {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(65_536, SimpleMoveGen, MaterialEvaluator),
            current_pos: Position::default(),
            limits: GoLimits::default(),
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn run(&mut self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let cmd = line.unwrap();
            match cmd.as_str() {
                "uci" => self.handle_uci(&mut stdout),
                "isready" => self.handle_isready(&mut stdout),
                "ucinewgame" => self.handle_newgame(&mut stdout),
                cmd if cmd.starts_with("position") => self.handle_position(cmd, &mut stdout),
                cmd if cmd.starts_with("go") => self.handle_go(cmd, &mut stdout),
                "stop" => {
                    self.stop.store(true, Ordering::Relaxed);
                }
                cmd if cmd.starts_with("bench") => self.handle_bench(cmd, &mut stdout),
                "quit" => break,
                _ => {}
            }
            stdout.flush().unwrap();
        }
    }

    fn handle_uci(&self, out: &mut impl Write) {
        writeln!(out, "id name Cody").unwrap();
        writeln!(out, "id author Strong Noodle").unwrap();
        writeln!(out, "uciok").unwrap();
    }

    fn handle_isready(&self, out: &mut impl Write) {
        writeln!(out, "readyok").unwrap();
    }

    fn handle_position(&mut self, cmd: &str, _out: &mut impl Write) {
        let mut tokens = cmd.split_whitespace().skip(1).peekable();
        let mut pos = Position::default();

        if let Some(tok) = tokens.peek().copied() {
            match tok {
                "startpos" => {
                    tokens.next(); // consume "startpos"
                    pos = Position::default();
                }
                "fen" => {
                    tokens.next(); // consume "fen"
                    let fen_parts: Vec<&str> =
                        tokens.by_ref().take_while(|&t| t != "moves").collect();
                    let fen_str = fen_parts.join(" ");
                    pos = Position::from_fen(&fen_str);
                }
                _ => {
                    // Unknown format: keep default start position
                }
            }
        }

        // Apply subsequent moves if present
        if let Some(&"moves") = tokens.peek() {
            tokens.next(); // consume "moves"
            for mv in tokens {
                match pos.parse_uci_move(mv) {
                    Some(chess_move) => {
                        // Apply move into pos, in-place or via a helper you have
                        let mut new_pos = Position::default();
                        pos.apply_move_into(&chess_move, &mut new_pos);
                        pos = new_pos;
                    }
                    None => {
                        // If you want visibility during debugging:
                        dbg!("Failed to parse UCI move", mv, &pos);
                        // Per UCI, silently ignoring is acceptable, but better to get this right.
                    }
                }
            }
        }

        // Persist the new position
        self.current_pos = pos;
    }

    fn handle_go(&mut self, cmd: &str, out: &mut impl Write) {
        self.stop.store(false, Ordering::Relaxed);
        self.limits = self.parse_go_limits(cmd);

        let start = std::time::Instant::now();
        let max_depth = self.limits.depth.unwrap_or(64);
        NODE_COUNT.store(0, Ordering::Relaxed);

        let mut last_completed_move = None;

        for d in 1..=max_depth {
            let (bm, sc) = self.engine.search(&self.current_pos, d);

            // Store the PV from this fully completed depth
            last_completed_move = Some(bm);

            let elapsed = start.elapsed().as_millis() as u64;
            let nodes = NODE_COUNT.load(Ordering::Relaxed);
            let nps = if elapsed > 0 {
                nodes * 1000 / elapsed
            } else {
                0
            };

            // Print info line with proper mate/centipawn formatting
            if sc.abs() > MATE_SCORE - 100 {
                let mate_in = if sc > 0 {
                    (MATE_SCORE - sc + 1) / 2
                } else {
                    -(MATE_SCORE + sc) / 2
                };
                writeln!(
                    out,
                    "info depth {} score mate {} nodes {} time {} nps {}",
                    d, mate_in, nodes, elapsed, nps
                )
                .unwrap();
            } else {
                writeln!(
                    out,
                    "info depth {} score cp {} nodes {} time {} nps {}",
                    d, sc, nodes, elapsed, nps
                )
                .unwrap();
            }

            out.flush().unwrap();

            // Stop conditions
            if let Some(mt) = self.limits.movetime_ms
                && elapsed >= mt
            {
                break;
            }
            if self.stop.load(Ordering::Relaxed) {
                break;
            }
        }

        // Output the best move from the last *completed* depth
        let bm = last_completed_move.expect("At least depth 1 should produce a move");
        let bm_str = if bm.is_null() {
            "0000".to_string()
        } else {
            bm.to_string()
        };
        writeln!(out, "bestmove {}", bm_str).unwrap();
    }

    fn parse_go_limits(&self, cmd: &str) -> GoLimits {
        let mut limits = GoLimits::default();
        let mut it = cmd.split_whitespace().skip(1).peekable();

        while let Some(tok) = it.next() {
            match tok {
                "depth" => limits.depth = it.next().and_then(|s| s.parse().ok()),
                "movetime" => limits.movetime_ms = it.next().and_then(|s| s.parse().ok()),
                "wtime" => limits.wtime_ms = it.next().and_then(|s| s.parse().ok()),
                "btime" => limits.btime_ms = it.next().and_then(|s| s.parse().ok()),
                "winc" => limits.winc_ms = it.next().and_then(|s| s.parse().ok()),
                "binc" => limits.binc_ms = it.next().and_then(|s| s.parse().ok()),
                "ponder" => limits.ponder = true,
                "infinite" => limits.infinite = true,
                _ => {}
            }
        }

        // Optional: if no depth/movetime but clocks are provided, allocate a sane per-move budget.
        // Example heuristic (very rough):
        if limits.depth.is_none()
            && limits.movetime_ms.is_none()
            && let (Some(wt), Some(bt)) = (limits.wtime_ms, limits.btime_ms)
        {
            // Pick side to move from the current position to decide which clock applies.
            // Roughly spend 1/30th of the remaining time + increment.
            let stm_white = self.current_pos.side_to_move == Color::White;
            let (my_time, my_inc) = if stm_white {
                (wt, limits.winc_ms.unwrap_or(0))
            } else {
                (bt, limits.binc_ms.unwrap_or(0))
            };
            let budget = (my_time / 30).max(5) + my_inc.min(100); // cap inc use a bit
            limits.movetime_ms = Some(budget);
        }

        limits
    }

    fn handle_bench(&mut self, _cmd: &str, out: &mut impl Write) {
        let depth = 4;

        // Clone into a Vec so we can sort
        let mut cases: Vec<&TestCase> = TEST_CASES.iter().collect();
        cases.sort_by(|a, b| a.name.cmp(b.name)); // alphabetical by name

        let mut total_nodes = 0u64;
        let start_all = std::time::Instant::now();

        // Precompute width for alignment
        let name_width = cases.iter().map(|tc| tc.name.len()).max().unwrap_or(0);

        for pos in cases {
            NODE_COUNT.store(0, Ordering::Relaxed);

            let start = std::time::Instant::now();
            let _score = self.engine.search(&pos.position(), depth);
            let elapsed = start.elapsed().as_secs_f64();
            let nodes = NODE_COUNT.load(Ordering::Relaxed);
            total_nodes += nodes;
            let nps = (nodes as f64 / elapsed) as u64;

            writeln!(out, "-----------------------------------------------").unwrap();
            writeln!(out, "{}", pos.fen).unwrap();
            writeln!(out, "Best move: {}", _score.0).unwrap();
            writeln!(
                out,
                "{:<width$}  nodes {:>10}  time {:>5}  nps {:>10}",
                pos.name,
                nodes,
                format!("{:.0}", elapsed * 1000.0),
                nps,
                width = name_width
            )
            .unwrap();

            out.flush().unwrap();
        }

        let total_time_ms = (start_all.elapsed().as_secs_f64() * 1000.0) as u64;
        let total_nps = (total_nodes as f64 / (total_time_ms as f64 / 1000.0)) as u64;

        writeln!(out, "===========================").unwrap();
        writeln!(out, "Total time (ms) : {}", total_time_ms).unwrap();
        writeln!(out, "Nodes searched  : {}", total_nodes).unwrap();
        writeln!(out, "Nodes/second    : {}", total_nps).unwrap();
        out.flush().unwrap();
    }

    fn handle_newgame(&mut self, _out: &mut impl Write) {
        self.current_pos = Position::default();
        self.limits = GoLimits::default();
        self.stop.store(false, Ordering::Relaxed);
        // If your engine has a TT/history, clear them:
        self.engine.clear_state();
    }
}

use bitboard::movegen::SimpleMoveGen;
use bitboard::piece::Color;
use bitboard::position::Position;
use engine::{Engine, MaterialEvaluator, NODE_COUNT, TEST_CASES, TestCase, VERBOSE};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::api::golimits::GoLimits;

pub struct CodyApi {
    engine: Engine<SimpleMoveGen, MaterialEvaluator>,
    current_pos: Position,
    limits: GoLimits,
    stop: Arc<AtomicBool>, // for future: stop support
    // Optional log file for UCI diagnostics (IN/OUT)
    log: Option<File>,
}

impl CodyApi {
    pub fn new() -> Self {
        let engine = Engine::new(65_536, SimpleMoveGen, MaterialEvaluator);
        // Try to open a log file in append mode; non-fatal if it fails.
        let log = OpenOptions::new()
            .create(true)
            .append(true)
            .open("cody_uci.log")
            .ok();

        Self {
            engine,
            current_pos: Position::default(),
            limits: GoLimits::default(),
            stop: Arc::new(AtomicBool::new(false)),
            log,
        }
    }

    // Use crate util for consistent millisecond-precision ISO timestamps.
    fn iso_stamp() -> String {
        engine::util::iso_stamp_ms()
    }

    pub fn run(&mut self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let cmd = line.unwrap();
            // Log incoming command
            self.log_in(&cmd);

            match cmd.as_str() {
                "uci" => self.handle_uci(&mut stdout),
                cmd if cmd.starts_with("setoption") => self.handle_setoption(cmd),
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

    fn log_in(&mut self, cmd: &str) {
        if let Some(f) = &mut self.log {
            let _ = writeln!(f, "{} IN: {}", Self::iso_stamp(), cmd);
        }
    }

    fn writeln_and_log(&mut self, out: &mut impl Write, text: &str) {
        writeln!(out, "{}", text).unwrap();
        if let Some(f) = &mut self.log {
            let _ = writeln!(f, "{} OUT: {}", Self::iso_stamp(), text);
        }
    }

    fn handle_uci(&mut self, out: &mut impl Write) {
        self.writeln_and_log(out, "id name Cody");
        self.writeln_and_log(out, "id author Strong Noodle");
        // Advertise a Threads option so UIs can control parallelism.
        let max_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let opt = format!(
            "option name Threads type spin default 1 min 1 max {}",
            max_threads
        );
        self.writeln_and_log(out, &opt);
        // Advertise Verbose option so UIs can toggle runtime verbose logging via setoption
        self.writeln_and_log(out, "option name Verbose type check default false");
        self.writeln_and_log(out, "uciok");
    }

    fn handle_setoption(&mut self, cmd: &str) {
        // Parse: setoption name <name...> value <value>
        let parts: Vec<&str> = cmd.split_whitespace().skip(1).collect();
        let mut name_idx: Option<usize> = None;
        let mut value_idx: Option<usize> = None;
        for (i, &p) in parts.iter().enumerate() {
            if p.eq_ignore_ascii_case("name") {
                name_idx = Some(i);
            } else if p.eq_ignore_ascii_case("value") {
                value_idx = Some(i);
            }
        }

        if let (Some(ni), Some(vi)) = (name_idx, value_idx)
            && ni < vi
        {
            let name = parts[ni + 1..vi].join(" ");
            let value = parts.get(vi + 1).copied().unwrap_or("");
            if name.eq_ignore_ascii_case("threads") {
                if let Ok(n) = value.parse::<usize>() {
                    self.engine.set_num_threads(n.max(1));
                }
            } else if name.eq_ignore_ascii_case("verbose") || name.eq_ignore_ascii_case("verbosE") {
                // Accept "true"/"false" (case-insensitive) to toggle runtime verbose logging.
                let enable = value.eq_ignore_ascii_case("true");
                VERBOSE.store(enable, Ordering::Relaxed);
            } else if name.eq_ignore_ascii_case("verbose") {
                // Fallback for oddly cased names
                let enable = value.eq_ignore_ascii_case("true");
                VERBOSE.store(enable, Ordering::Relaxed);
            }
        }
    }

    fn handle_isready(&mut self, out: &mut impl Write) {
        self.writeln_and_log(out, "readyok");
    }

    fn handle_position(&mut self, cmd: &str, out: &mut impl Write) {
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
                        // If you want visibility during debugging, write to the UCI log and stdout when verbose is enabled.
                        if VERBOSE.load(Ordering::Relaxed) {
                            let text =
                                format!("Failed to parse UCI move {} for pos {}", mv, pos.to_fen());
                            // Best-effort: write to stdout and append to cody_uci.log
                            self.writeln_and_log(out, &text);
                        }
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
        // Debug trace: announce parsed limits so UIs / logs can see we've started handling go
        if VERBOSE.load(Ordering::Relaxed) {
            self.writeln_and_log(out, &format!("debug: handle_go limits: {:?}", self.limits));
            out.flush().ok();
        }

        // Use engine's iterative deepening search. We forward the move time limit
        // and a stop flag so the engine can honor them between depths.
        let max_depth = self.limits.depth.unwrap_or(64);
        NODE_COUNT.store(0, Ordering::Relaxed);

        let time_budget = self.limits.movetime_ms;

        let (bm, _sc) =
            self.engine
                .search(&self.current_pos, max_depth, time_budget, Some(&*self.stop));

        let bm_str = if bm.is_null() {
            "0000".to_string()
        } else {
            bm.to_string()
        };
        self.writeln_and_log(out, &format!("bestmove {}", bm_str));
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

        // Fallback: if still no limits and not infinite, use a sensible default
        if limits.depth.is_none() && limits.movetime_ms.is_none() && !limits.infinite {
            limits.movetime_ms = Some(1000); // 1 second default for bare "go" command
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
            let _score = self.engine.search(&pos.position(), depth, None, None);
            let elapsed = start.elapsed().as_secs_f64();
            let nodes = NODE_COUNT.load(Ordering::Relaxed);
            total_nodes += nodes;
            let nps = (nodes as f64 / elapsed) as u64;

            self.writeln_and_log(out, "-----------------------------------------------");
            self.writeln_and_log(out, pos.fen);
            self.writeln_and_log(out, &format!("Best move: {}", _score.0));
            self.writeln_and_log(
                out,
                &format!(
                    "{:<width$}  nodes {:>10}  time {:>5}  nps {:>10}",
                    pos.name,
                    nodes,
                    format!("{:.0}", elapsed * 1000.0),
                    nps,
                    width = name_width
                ),
            );

            out.flush().unwrap();
        }

        let total_time_ms = (start_all.elapsed().as_secs_f64() * 1000.0) as u64;
        let total_nps = (total_nodes as f64 / (total_time_ms as f64 / 1000.0)) as u64;

        self.writeln_and_log(out, "===========================");
        self.writeln_and_log(out, &format!("Total time (ms) : {}", total_time_ms));
        self.writeln_and_log(out, &format!("Nodes searched  : {}", total_nodes));
        self.writeln_and_log(out, &format!("Nodes/second    : {}", total_nps));
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

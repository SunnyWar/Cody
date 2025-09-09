use cody::{Engine, MaterialEvaluator, NODE_COUNT, Position, SimpleMoveGen, TEST_POSITIONS};
use std::io::{self, BufRead, Write};
use std::sync::atomic::Ordering;

pub struct CodyApi {
    engine: Engine<SimpleMoveGen, MaterialEvaluator>,
}

impl CodyApi {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(65536, SimpleMoveGen, MaterialEvaluator),
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
        // Strip the leading "position" keyword
        let mut tokens = cmd.split_whitespace().skip(1).peekable();

        let pos = if let Some(tok) = tokens.peek() {
            match *tok {
                "startpos" => {
                    tokens.next(); // consume "startpos"
                    Position::default() // your default() should be the standard start position
                }
                "fen" => {
                    tokens.next(); // consume "fen"
                    // Collect FEN parts until we hit "moves" or run out
                    let fen_parts: Vec<&str> =
                        tokens.by_ref().take_while(|&t| t != "moves").collect();
                    let fen_str = fen_parts.join(" ");
                    Position::from_fen(&fen_str)
                }
                _ => {
                    // Unknown format — fall back to default
                    Position::default()
                }
            }
        } else {
            Position::default()
        };

        // If there’s a "moves" section, apply them in order
        if let Some(&"moves") = tokens.peek() {
            tokens.next(); // consume "moves"
            for mv in tokens {
                // Assuming you have a way to parse UCI move strings into your Move type
                if let Some(parsed) = pos.parse_uci_move(mv) {
                    // TODO
                }
            }
        }

        // Store the position in your engine state
        // TODO
    }

    fn handle_go(&mut self, cmd: &str, out: &mut impl Write) {
        // Default depth if none specified
        let mut depth = 5;

        // Parse "depth N" from the UCI go command
        let tokens: Vec<&str> = cmd.split_whitespace().collect();
        if let Some(i) = tokens.iter().position(|&t| t == "depth")
            && let Some(d_str) = tokens.get(i + 1)
            && let Ok(d_val) = d_str.parse::<usize>()
        {
            depth = d_val;
        }

        // TODO: replace with actual parsed position from "position" command
        let pos = Position::default();

        let start = std::time::Instant::now();
        let (best_move, score) = self.engine.search(&pos, depth);
        let elapsed = start.elapsed().as_secs_f64();
        let nodes = NODE_COUNT.load(Ordering::Relaxed);
        let nps = (nodes as f64 / elapsed) as u64;

        writeln!(
            out,
            "info depth {} score cp {} nodes {} time {} nps {}",
            depth,
            score,
            nodes,
            (elapsed * 1000.0) as u64,
            nps
        )
        .unwrap();

        writeln!(out, "bestmove {}", best_move).unwrap();
    }

    fn handle_bench(&mut self, _cmd: &str, out: &mut impl Write) {
        let depth = 4;

        let mut total_nodes = 0u64;
        let start_all = std::time::Instant::now();

        for pos in TEST_POSITIONS.iter() {
            NODE_COUNT.store(0, Ordering::Relaxed);

            let start = std::time::Instant::now();
            let _score = self.engine.search(pos, depth);
            let elapsed = start.elapsed().as_secs_f64();
            let nodes = NODE_COUNT.load(Ordering::Relaxed);
            total_nodes += nodes;
            let nps = (nodes as f64 / elapsed) as u64;

            writeln!(
                out,
                "benchpos nodes {} time {:.0} nps {}",
                nodes,
                elapsed * 1000.0,
                nps
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
        // No output required by UCI.
        // If all state is reset in `handle_go`, nothing to do here.
    }
}

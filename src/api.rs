use cody::{Engine, MaterialEvaluator, NODE_COUNT, Position, SimpleMoveGen};
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

    fn handle_position(&mut self, _cmd: &str, _out: &mut impl Write) {
        // TODO: parse FEN or "startpos" and set position
    }

    fn handle_go(&mut self, _cmd: &str, out: &mut impl Write) {
        let depth = 5; // TODO: parse from command
        let pos = Position::default(); // TODO: use parsed position

        let start = std::time::Instant::now();
        let score = self.engine.search(&pos, depth);
        let elapsed = start.elapsed().as_secs_f64();
        let nodes = NODE_COUNT.load(Ordering::Relaxed);
        let nps = (nodes as f64 / elapsed) as u64;

        writeln!(
            out,
            "info depth {} nodes {} time {} nps {}",
            depth,
            nodes,
            (elapsed * 1000.0) as u64,
            nps
        )
        .unwrap();
        writeln!(out, "bestmove e2e4").unwrap(); // TODO: actual best move
    }

    fn handle_bench(&mut self, _cmd: &str, out: &mut impl Write) {
        let depth = 5;
        let test_positions = vec![
            Position::default(),
            // TODO: Add more FENs here
        ];

        let mut total_nodes = 0u64;
        let start_all = std::time::Instant::now();

        for pos in &test_positions {
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

        let total_time = start_all.elapsed().as_secs_f64();
        let total_nps = (total_nodes as f64 / total_time) as u64;
        writeln!(
            out,
            "bench total_nodes {} total_time {:.0} total_nps {}",
            total_nodes,
            total_time * 1000.0,
            total_nps
        )
        .unwrap();
        out.flush().unwrap();
    }
}

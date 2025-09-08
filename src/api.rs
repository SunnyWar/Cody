//src/api.rs
use std::io::{self, BufRead, Write};
use std::sync::atomic::Ordering;
use cody::{Engine, MaterialEvaluator, Position, SimpleMoveGen, NODE_COUNT};

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
                "uci" => {
                    writeln!(stdout, "id name Cody").unwrap();
                    writeln!(stdout, "id author David").unwrap();
                    writeln!(stdout, "uciok").unwrap();
                }
                "isready" => {
                    writeln!(stdout, "readyok").unwrap();
                }
                cmd if cmd.starts_with("position") => {
                    // TODO: parse FEN or "startpos" and set position
                }
                cmd if cmd.starts_with("go") => {
                    let depth = 5; // TODO: parse from command
                    let pos = Position::default(); // TODO: use parsed position

                    let start = std::time::Instant::now();
                    let score = self.engine.search(&pos, depth);
                    let elapsed = start.elapsed().as_secs_f64();
                    let nodes = NODE_COUNT.load(Ordering::Relaxed);
                    let nps = (nodes as f64 / elapsed) as u64;

                    writeln!(
                        stdout,
                        "info depth {} nodes {} time {} nps {}",
                        depth,
                        nodes,
                        (elapsed * 1000.0) as u64,
                        nps
                    ).unwrap();
                    writeln!(stdout, "bestmove e2e4").unwrap(); // TODO: actual best move
                }
                "quit" => break,
                _ => {}
            }
            stdout.flush().unwrap();
        }
    }
}

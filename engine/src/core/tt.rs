use crate::search::core::INF;
use bitboard::mov::ChessMove;

#[derive(Clone, Copy, Debug)]
pub enum TTFlag {
    Exact = 0,
    Lower = 1,
    Upper = 2,
}

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub key: u64,
    pub value: i32,
    pub depth: i8,
    pub flag: u8,
    pub best_move: ChessMove,
}

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry {
            key: 0,
            value: 0,
            depth: -1,
            flag: TTFlag::Upper as u8,
            best_move: ChessMove::null(),
        }
    }
}

pub struct TranspositionTable {
    pub entries: Vec<TTEntry>,
    mask: usize,
}

impl TranspositionTable {
    pub fn new(size_pow2: usize) -> Self {
        let cap = 1usize << size_pow2;
        TranspositionTable {
            entries: vec![TTEntry::default(); cap],
            mask: cap - 1,
        }
    }

    fn index(&self, key: u64) -> usize {
        (key as usize) & self.mask
    }

    pub fn clear(&mut self) {
        for e in self.entries.iter_mut() {
            *e = TTEntry::default();
        }
    }

    pub fn probe(&self, key: u64, depth: i8, alpha: i32, beta: i32) -> Option<TTEntry> {
        let idx = self.index(key);
        let e = self.entries[idx];
        if e.key != key {
            return None;
        }
        if e.depth < depth {
            return None; // stored value from shallower search; ignore
        }

        match e.flag {
            x if x == TTFlag::Exact as u8 => Some(e),
            x if x == TTFlag::Lower as u8 => {
                if e.value >= beta {
                    Some(e)
                } else {
                    None
                }
            }
            x if x == TTFlag::Upper as u8 => {
                if e.value <= alpha {
                    Some(e)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn store(&mut self, key: u64, value: i32, depth: i8, flag: TTFlag, best_move: ChessMove) {
        let idx = self.index(key);
        let mut e = self.entries[idx];
        // Replacement: prefer deeper entries
        if e.depth > depth {
            // keep existing deeper entry
            return;
        }
        e.key = key;
        e.value = value;
        e.depth = depth;
        e.flag = flag as u8;
        e.best_move = best_move;
        self.entries[idx] = e;
    }
}

use bitboard::mov::ChessMove;

#[derive(Clone, Copy, Debug)]
pub enum TTFlag {
    Exact = 0,
    Lower = 1,
    Upper = 2,
}

#[derive(Clone, Copy, Debug)]
#[repr(align(32))]
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

    pub fn clear(&mut self) {
        for e in &mut self.entries {
            *e = TTEntry::default();
        }
    }

    #[must_use]
    pub fn probe(&self, key: u64, depth: i8, alpha: i32, beta: i32) -> Option<TTEntry> {
        #[allow(clippy::cast_possible_truncation)]
        let idx = (key as usize) & self.mask;
        let e = self.entries[idx];

        if e.key != key {
            return None;
        }
        if e.depth < depth {
            return None; // stored value from shallower search; ignore
        }

        // Optimize flag checks with direct equality comparisons
        let flag = e.flag;
        if flag == TTFlag::Exact as u8 {
            return Some(e);
        }
        if flag == TTFlag::Lower as u8 {
            return if e.value >= beta { Some(e) } else { None };
        }
        if flag == TTFlag::Upper as u8 {
            return if e.value <= alpha { Some(e) } else { None };
        }
        None
    }

    pub fn store(&mut self, key: u64, value: i32, depth: i8, flag: TTFlag, best_move: ChessMove) {
        #[allow(clippy::cast_possible_truncation)]
        let idx = (key as usize) & self.mask;
        let e = &mut self.entries[idx];
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
    }

    /// Approximate hash occupancy in per-mille, similar to UCI `hashfull`.
    ///
    /// We sample up to the first 1000 entries to keep this cheap enough for
    /// periodic info reporting.
    #[must_use]
    pub fn hashfull_per_mille(&self) -> u16 {
        let sample_size = self.entries.len().min(1000);
        if sample_size == 0 {
            return 0;
        }

        let used = self
            .entries
            .iter()
            .take(sample_size)
            .filter(|e| e.key != 0)
            .count();

        #[allow(clippy::cast_possible_truncation)]
        let result = ((used * 1000) / sample_size) as u16;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::TTFlag;
    use super::TranspositionTable;
    use bitboard::mov::ChessMove;

    #[test]
    fn test_probe_exact_returns_entry() {
        let mut tt = TranspositionTable::new(4);
        let key = 0x1234_5678_9abc_def0;
        let bm = ChessMove::null();
        tt.store(key, 42, 6, TTFlag::Exact, bm);

        let hit = tt.probe(key, 4, -100, 100).expect("expected exact hit");
        assert_eq!(hit.value, 42);
        assert_eq!(hit.flag, TTFlag::Exact as u8);
    }

    #[test]
    fn test_probe_lower_respects_beta_cutoff() {
        let mut tt = TranspositionTable::new(4);
        let key = 0x1111_2222_3333_4444;
        tt.store(key, 250, 5, TTFlag::Lower, ChessMove::null());

        assert!(tt.probe(key, 5, -100, 200).is_some());
        assert!(tt.probe(key, 5, -100, 300).is_none());
    }

    #[test]
    fn test_probe_upper_respects_alpha_cutoff() {
        let mut tt = TranspositionTable::new(4);
        let key = 0x9999_aaaa_bbbb_cccc;
        tt.store(key, -180, 5, TTFlag::Upper, ChessMove::null());

        assert!(tt.probe(key, 5, -100, 100).is_some());
        assert!(tt.probe(key, 5, -250, 100).is_none());
    }

    #[test]
    fn test_store_keeps_deeper_entry() {
        let mut tt = TranspositionTable::new(2);
        let key = 0x55aa_55aa_55aa_55aa;
        tt.store(key, 10, 8, TTFlag::Exact, ChessMove::null());
        tt.store(key, 99, 4, TTFlag::Exact, ChessMove::null());

        let hit = tt
            .probe(key, 8, -1_000, 1_000)
            .expect("expected stored entry");
        assert_eq!(hit.value, 10);
        assert_eq!(hit.depth, 8);
    }
}

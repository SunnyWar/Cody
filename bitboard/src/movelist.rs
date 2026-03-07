// Stack-allocated move list to eliminate heap allocations in hot path
// Maximum legal moves in any chess position is 218
// We use 256 for power-of-2 alignment and safety margin

use crate::mov::ChessMove;

const MAX_MOVES: usize = 256;

#[derive(Clone)]
pub struct MoveList {
    moves: [ChessMove; MAX_MOVES],
    len: usize,
}

impl MoveList {
    #[inline]
    pub fn new() -> Self {
        Self {
            moves: [ChessMove::null(); MAX_MOVES],
            len: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, mv: ChessMove) {
        debug_assert!(self.len < MAX_MOVES, "MoveList overflow");
        self.moves[self.len] = mv;
        self.len += 1;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    pub fn as_slice(&self) -> &[ChessMove] {
        &self.moves[..self.len]
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [ChessMove] {
        &mut self.moves[..self.len]
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &ChessMove> {
        self.moves[..self.len].iter()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&ChessMove> {
        if index < self.len {
            Some(&self.moves[index])
        } else {
            None
        }
    }

    /// Swap two moves by index (used for move ordering)
    #[inline]
    pub fn swap(&mut self, a: usize, b: usize) {
        if a < self.len && b < self.len {
            self.moves.swap(a, b);
        }
    }

    /// Convert to Vec for compatibility with existing code
    /// This allocates but allows gradual migration
    #[inline]
    pub fn to_vec(&self) -> Vec<ChessMove> {
        self.moves[..self.len].to_vec()
    }

    /// Create from Vec (for tests and compatibility)
    #[inline]
    pub fn from_vec(vec: Vec<ChessMove>) -> Self {
        let mut list = Self::new();
        for mv in vec {
            list.push(mv);
        }
        list
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Index to allow list[i] syntax
impl std::ops::Index<usize> for MoveList {
    type Output = ChessMove;

    fn index(&self, index: usize) -> &Self::Output {
        &self.moves[index]
    }
}

impl std::ops::IndexMut<usize> for MoveList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.moves[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Square;
    use crate::mov::MoveType;

    #[test]
    fn test_movelist_basic() {
        let mut list = MoveList::new();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());

        let mv = ChessMove::new(Square::E2, Square::E4, MoveType::Quiet);
        list.push(mv);
        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());
        assert_eq!(list[0], mv);
    }

    #[test]
    fn test_movelist_iteration() {
        let mut list = MoveList::new();
        list.push(ChessMove::new(Square::E2, Square::E4, MoveType::Quiet));
        list.push(ChessMove::new(Square::D2, Square::D4, MoveType::Quiet));

        let mut count = 0;
        for _mv in list.iter() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_movelist_swap() {
        let mut list = MoveList::new();
        let mv1 = ChessMove::new(Square::E2, Square::E4, MoveType::Quiet);
        let mv2 = ChessMove::new(Square::D2, Square::D4, MoveType::Quiet);

        list.push(mv1);
        list.push(mv2);
        list.swap(0, 1);

        assert_eq!(list[0], mv2);
        assert_eq!(list[1], mv1);
    }
}

# Parallel expansion architecture for Cody

A design for a cache-friendly, allocation-free search that scales from single-thread to multi-thread with minimal disruption.

---

## Goals and constraints

- **Primary goal:** Max throughput on move generation and child expansion with excellent cache locality and zero heap allocation in the hot path.
- **Secondary goal:** Maintain simple, enforceable invariants to avoid index misuse and aliasing bugs.
- **Scalability:** Start single-threaded for correctness, then scale to multi-thread with work-stealing.
- **Rust-first:** Strongly typed interfaces, minimal unsafe, clear ownership boundaries.
- **Crates:** Prefer mature, high-performance crates (e.g., crossbeam, parking_lot, smallvec) only where they deliver measurable benefit.

---

## High-level architecture

- **Per-thread arenas:** Each worker owns a contiguous arena of Node slots. No sharing; no locks. This keeps writes local to a core and avoids false sharing.
- **Task queues with work stealing:** Tasks represent units of expansion work. Workers pop a task, generate children, and push more tasks. Use per-thread deques with work stealing for load balance.
- **Two-phase pipeline:**
  - **Expansion phase (parallel):** Expand nodes, produce terminal leaves.
  - **Evaluation phase (single-threaded or batched):** Evaluate terminal leaves and aggregate scores.
- **Data movement:** Positions are applied in-place into fresh arena slots; tasks carry only indices and shallow metadata.

---

## Core data structures and APIs

### Node arena (per-thread)

A fast, contiguous, reuse-friendly arena. Single writer, no synchronization required.

```rust
pub struct Arena {
    nodes: Vec<Node>,
    next_free: usize,
    free_list: Vec<usize>, // LIFO reuse for locality
}

impl Arena {
    pub fn with_capacity(capacity: usize) -> Self { /* prefill or reserve */ }

    // O(1), branch-predictable
    #[inline]
    pub fn alloc(&mut self) -> Option<usize> {
        if let Some(idx) = self.free_list.pop() { return Some(idx); }
        if self.next_free < self.nodes.len() {
            let idx = self.next_free;
            self.next_free += 1;
            Some(idx)
        } else {
            None
        }
    }

    #[inline]
    pub fn free(&mut self, idx: usize) {
        debug_assert!(idx < self.next_free);
        self.free_list.push(idx);
    }

    #[inline]
    pub fn get(&self, idx: usize) -> &Node { &self.nodes[idx] }

    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> &mut Node { &mut self.nodes[idx] }

    // Safe pair accessors for common patterns; assert distinct indices
    #[inline]
    pub fn pair<'a>(&'a self, a: usize, b: usize) -> (&'a Node, &'a Node) { /* ... */ }

    #[inline]
    pub fn pair_mut<'a>(&'a mut self, a: usize, b: usize) -> (&'a mut Node, &'a mut Node) { /* ... */ }
}
```

- **Invariant:** Only the owning thread touches its `Arena`.
- **Capacity:** Size to peak concurrent subtree for one worker. Track high-water mark to right-size.

### Task

The unit of work. No heavy data, just references to arena slots and search metadata.

```rust
#[derive(Clone, Copy)]
pub struct Task {
    pub worker_id: u32,        // which arena
    pub node_idx: u32,         // index in that arena
    pub depth_remaining: u16,  // remaining plies
    pub ply: u16,              // absolute ply for mate distance, etc.
    // Optional: flags (checks, extensions), parent linkage, move used
}
```

### Terminal leaf

Positions that require evaluation or are terminal outcomes.

```rust
pub struct Leaf {
    pub worker_id: u32,
    pub node_idx: u32,
    pub ply: u16,
    pub terminal_kind: TerminalKind, // Mate/Stalemate/Depth
}
```

---

## Execution model

### Single-threaded mode (bootstrap)

- **Queue:** Simple VecDeque<Task> on one thread.
- **Arena:** One local Arena.
- **Loop:** Pop → expand → push children or record Leaf → continue.
- **Eval:** After queue drains, evaluate all leaves and back-propagate or select best move.

This proves correctness using the same Task/Arena abstractions that will be used in parallel.

### Multi-threaded mode

- **Queues:** Per-thread deques with work stealing.
- **Arbiter:** Simple controller seeds root tasks; workers run until global quiescence.
- **Termination detection:** Atomic “in-flight” counter: increment on push of non-terminal tasks, decrement when a task is fully processed. When counter hits zero and all deques are empty, expansion ends.
- **Final evaluation:** Single-thread (or a small pool) consumes per-thread leaf buffers and computes evaluation.

---

## Concurrency design

### Work queues

- **Preferred:** crossbeam_deque (work-stealing deques).
  - **Worker API:** push/pop at the local end (fast, LIFO).
  - **Steal API:** other threads steal from the opposite end (FIFO-ish to preserve locality).
- **Alternative:** crossbeam::queue::SegQueue for simplicity; expect higher contention.

### Per-thread state

```rust
pub struct Worker {
    pub id: usize,
    pub arena: Arena,
    pub deque: crossbeam_deque::Worker<Task>,
    pub leaf_buf: Vec<Leaf>,           // thread-local buffer, later merged
    pub rng: SmallRng,                 // if randomness is needed
}
```

- **No sharing:** Workers never touch each other’s arenas.
- **Batching:** Push children in small batches to amortize queue overhead.

### Shared coordination

- **Stealers:** Vec<crossbeam_deque::Stealer<Task>> shared among threads.
- **In-flight counter:** AtomicUsize to track active expansion tasks.
- **Stop flag:** AtomicBool for time controls or external stop requests.

---

## Search algorithm mapping

- **Expansion task handling:**
  - **Inputs:** Task with (worker_id, node_idx, depth_remaining, ply).
  - **Process:**
    - Generate moves from `arena.get(node_idx)`.
    - For each move:
      - If terminal (checkmate, stalemate, or depth limit): append a Leaf.
      - Else: alloc a child slot in local arena, apply move into child, enqueue Task for child.
- **Evaluation phase:**
  - Iterate leaves; compute scores.
  - If needed, propagate to parents via separate structure (e.g., a result map keyed by parent ref) or simply select best at root when your goal is move choice rather than full PV reconstruction.

Note: If your final objective is best move at the root, you can tag tasks by root-move bucket and aggregate leaf scores per root-move, avoiding full parent linkage.

---

## Diagnostics and safety

- **Index safety:** Indexing is encapsulated inside Arena; external code never computes `ply+1` or similar.
- **Capacity assertions:** Arena asserts on overflow; expose `high_water_mark()` for sizing.
- **Counters:** Nodes expanded, tasks pushed, steals performed, arena usage (peak and current).
- **Poison checks:** Optional debug mode “generation” counters per slot to catch stale reuse.
- **Determinism mode:** Optional single-thread deterministic mode for reproducible tests.

---

## Implementation plan

### Phase 0: Prep

- **Label:** Internal refactor
- **Tasks:**
  - **Extract Node and Position helpers:** Ensure `apply_move_into` is zero-copy and branch-friendly.
  - **Define Task and Leaf types:** Keep them `Copy` and small.

### Phase 1: Single-threaded prototype

- **Build Arena v1:**
  - **Features:** `with_capacity`, `alloc`, `free`, `get`, `get_mut`, `high_water_mark`.
  - **Capacity:** `max_nodes_single_thread` (e.g., depth-limited worst case).
- **Build sequential queue loop:**
  - **Queue:** `VecDeque<Task>`.
  - **Seed:** Root position in arena slot 0; push initial Task.
  - **Expand:** Pop, generate, alloc children, push or leaf.
  - **Evaluate:** After drain, run evaluator over leaves.
- **Parity check:** Compare results with current DFS for small depths. Add assertions on counts.

### Phase 2: Harden interfaces

- **Encapsulate Arena access:** Hide raw indices behind an accessor trait in the search module.
- **Add diagnostics:** High-water mark, task counters, optional poison checks behind `cfg!(debug_assertions)`.
- **Bench:** Micro-benchmark expansion throughput vs. current DFS loop at equal depth.

### Phase 3: Parallel runtime

- **Integrate crossbeam_deque:**
  - **Crate:** crossbeam-deque (work-stealing).
  - **Create N workers:** One per logical core or tuned count.
  - **Per-thread arenas:** Size using single-thread peak × factor.
  - **In-flight counter and termination:** Atomics; barrier to end expansion.
- **Leaf handling:**
  - **Per-thread leaf buffers:** `Vec<Leaf>`; merge at end to avoid contention.
  - **Evaluation:** Single-thread or rayon small pool if needed (only if eval becomes visible in profile).
- **Steal tuning:**
  - **Batch steals:** Steal multiple tasks where supported or perform local batching for pushes.

### Phase 4: Validation and profiling

- **Correctness tests:**
  - **Perft-style tests:** Expand-only counts at fixed depths to compare with DFS baseline.
  - **Mate/stalemate edges:** Ensure terminal classification parity.
- **Profiling:**
  - **Measure:** Nodes/sec, steals/sec, queue contention, high-water marks.
  - **NUMA pinning:** Optional core pinning; ensure arenas are allocated post-thread-start to favor local NUMA nodes on big machines.

### Phase 5: Integrate with engine front-end

- **UCI integration:**
  - **Search limits:** Depth/time nodes.
  - **Stop flag:** Atomic stop on GUI command.
  - **Result selection:** Best move selection from aggregated root-move buckets.

---

## Code skeletons

### Arena

```rust
pub struct Arena {
    nodes: Vec<Node>,
    next_free: usize,
    free_list: Vec<usize>,
    high_water: usize,
}

impl Arena {
    pub fn with_capacity(cap: usize) -> Self {
        // Prefer pre-initialized Nodes to avoid branches in hot path
        Self {
            nodes: vec![Node::default(); cap],
            next_free: 0,
            free_list: Vec::new(),
            high_water: 0,
        }
    }

    #[inline]
    pub fn alloc(&mut self) -> Option<usize> {
        if let Some(i) = self.free_list.pop() {
            return Some(i);
        }
        if self.next_free < self.nodes.len() {
            let i = self.next_free;
            self.next_free += 1;
            self.high_water = self.high_water.max(self.next_free);
            Some(i)
        } else {
            None
        }
    }

    #[inline]
    pub fn free(&mut self, idx: usize) {
        debug_assert!(idx < self.next_free);
        self.free_list.push(idx);
    }

    #[inline]
    pub fn get(&self, idx: usize) -> &Node { &self.nodes[idx] }

    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> &mut Node { &mut self.nodes[idx] }

    #[inline]
    pub fn high_water_mark(&self) -> usize { self.high_water }
}
```

### Worker and scheduler (skeleton)

```rust
use crossbeam_deque::{Injector, Steal, Stealer, Worker as DequeWorker};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct Worker {
    pub id: usize,
    pub arena: Arena,
    pub local: DequeWorker<Task>,
    pub stealers: Vec<Stealer<Task>>,
    pub leaves: Vec<Leaf>,
}

impl Worker {
    pub fn run(&mut self, global: &Injector<Task>, inflight: &AtomicUsize, stop: &AtomicBool) {
        while !stop.load(Ordering::Relaxed) {
            if let Some(task) = self.pop_task(global) {
                self.process(task, global, inflight);
                if inflight.fetch_sub(1, Ordering::AcqRel) == 1 {
                    break; // last task completed
                }
            } else {
                // no work found anywhere
                if inflight.load(Ordering::Acquire) == 0 { break; }
                std::hint::spin_loop();
            }
        }
    }

    #[inline]
    fn pop_task(&mut self, global: &Injector<Task>) -> Option<Task> {
        if let Some(t) = self.local.pop() { return Some(t); }
        loop {
            match global.steal_batch_and_pop(&self.local) {
                Steal::Success(t) => return Some(t),
                Steal::Retry => continue,
                Steal::Empty => break,
            }
        }
        for s in &self.stealers {
            match s.steal() {
                Steal::Success(t) => return Some(t),
                Steal::Retry => continue,
                Steal::Empty => (),
            }
        }
        None
    }

    fn process(&mut self, task: Task, global: &Injector<Task>, inflight: &AtomicUsize) {
        let node = self.arena.get(task.node_idx as usize);
        let moves = generate_moves(&node.position); // your movegen

        if moves.is_empty() || task.depth_remaining == 0 {
            self.leaves.push(Leaf { worker_id: self.id as u32, node_idx: task.node_idx, ply: task.ply, terminal_kind: classify_terminal(node, &moves, task.depth_remaining) });
            return;
        }

        // Expand children, possibly push in batches
        let mut pushed = 0usize;
        for m in moves {
            let child_idx = match self.arena.alloc() {
                Some(i) => i,
                None => { /* fallback: spill to a local overflow buffer or truncate depth */ break; }
            };
            {
                let (parent, child) = (self.arena.get(task.node_idx as usize), self.arena.get_mut(child_idx));
                parent.position.apply_move_into(&m, &mut child.position);
            }
            let child = Task {
                worker_id: self.id as u32,
                node_idx: child_idx as u32,
                depth_remaining: task.depth_remaining - 1,
                ply: task.ply + 1,
            };
            self.local.push(child);
            pushed += 1;
        }
        if pushed > 0 {
            inflight.fetch_add(pushed, Ordering::AcqRel);
        }
        // Free parent if you won’t revisit it (optional, depends on result needs)
        // self.arena.free(task.node_idx as usize);
    }
}
```

---

## Testing and benchmarking

- **Correctness tests:**
  - **Perft-equivalent counts:** Validate node counts per depth against DFS, with and without check detection.
  - **Terminal cases:** Checkmates, stalemates, insufficient material.
  - **Repeatability:** Deterministic single-thread run should be stable across seeds.
- **Performance tests:**
  - **Microbench:** Nodes/sec for expansion only (disable eval).
  - **Throughput scaling:** 1, 2, 4, 8 threads; measure speedup and contention.
  - **Arena sizing:** Record high-water mark across positions and depths; right-size arenas.
- **Instrumentation:**
  - **Counters:** tasks pushed, steals, queue retries, arena allocs/frees, high-water.
  - **Timing:** Use std::time::Instant around phases; keep it lean to avoid perturbing caches.

---

## Future extensions

- **Move ordering at source:** Lightweight heuristics (e.g., captures, checks first) before pushing children.
- **Transposition-aware expansion:** Integrate a read-only TT probe during expansion to skip dominated nodes.
- **Hybrid eval:** If eval becomes non-trivial, parallelize evaluation with a small worker pool and bounded queue.
- **NUMA pinning:** Pin threads and allocate arenas post-pin to capture local memory.
- **Overflow strategy:** If arena runs out, optionally spill to a secondary pool or truncate depth gracefully with a debug log.

---

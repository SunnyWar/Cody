Here’s the updated **type‑safe, enum‑heavy, reference‑friendly** architecture document in `.md` format, ready to drop into your project.  
I’ve kept the structure from the earlier design, but replaced all raw integers with semantic newtypes/enums, and made the single‑thread bootstrap phase use references where possible.  

---

```markdown
# Cody Parallel Expansion Architecture (Type-Safe Edition)

A design for a cache-friendly, allocation-free search that scales from single-thread to multi-thread with minimal disruption, while maximizing Rust's type safety.

---

## Goals and Constraints

- **Primary goal:** Max throughput on move generation and child expansion with excellent cache locality and zero heap allocation in the hot path.
- **Secondary goal:** Enforce invariants at compile time using Rust's type system (newtypes, enums, lifetimes).
- **Scalability:** Start single-threaded for correctness, then scale to multi-thread with work-stealing.
- **Rust-first:** Strongly typed interfaces, minimal `unsafe`, clear ownership boundaries.
- **Crates:** Prefer mature, high-performance crates (`crossbeam`, `parking_lot`, `smallvec`) only where they deliver measurable benefit.

---

## Core Semantic Types

```rust
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct WorkerId(pub u8);

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct NodeId(pub u32);

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct Ply(pub u16);

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct Depth(pub u16);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TerminalKind {
    Checkmate,
    Stalemate,
    DepthLimit,
}
```

- **Zero-cost**: `#[repr(transparent)]` ensures no runtime overhead.
- **Compile-time safety**: Prevents mixing unrelated concepts (e.g., passing a `Depth` where a `NodeId` is expected).

---

## Arena API (Per-Thread)

```rust
pub struct Arena {
    nodes: Vec<Node>,
    next_free: usize,
    free_list: Vec<usize>,
    high_water: usize,
}

impl Arena {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            nodes: vec![Node::default(); cap],
            next_free: 0,
            free_list: Vec::new(),
            high_water: 0,
        }
    }

    #[inline]
    pub fn alloc(&mut self) -> Option<NodeId> {
        if let Some(idx) = self.free_list.pop() {
            return Some(NodeId(idx as u32));
        }
        if self.next_free < self.nodes.len() {
            let idx = self.next_free;
            self.next_free += 1;
            self.high_water = self.high_water.max(self.next_free);
            Some(NodeId(idx as u32))
        } else {
            None
        }
    }

    #[inline]
    pub fn free(&mut self, id: NodeId) {
        self.free_list.push(id.0 as usize);
    }

    #[inline]
    pub fn get(&self, id: NodeId) -> &Node {
        &self.nodes[id.0 as usize]
    }

    #[inline]
    pub fn get_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.0 as usize]
    }

    #[inline]
    pub fn high_water_mark(&self) -> usize {
        self.high_water
    }
}
```

- **Single-thread bootstrap**: Can use `&Node`/`&mut Node` directly.
- **Multi-thread**: Uses `NodeId` to pass work between threads safely.

---

## Task and Leaf Types

```rust
#[derive(Copy, Clone, Debug)]
pub struct Task {
    pub worker: WorkerId,
    pub node: NodeId,
    pub depth: Depth,
    pub ply: Ply,
}

pub struct Leaf {
    pub worker: WorkerId,
    pub node: NodeId,
    pub ply: Ply,
    pub terminal_kind: TerminalKind,
}
```

---

## Execution Model

### Single-threaded Mode (Bootstrap)

- **Queue:** `VecDeque<Task<'a>>` where `Task` holds `&Node` or `&mut Node`.
- **Arena:** One local `Arena`.
- **Loop:** Pop → expand → push children or record `Leaf` → continue.
- **Eval:** After queue drains, evaluate all leaves.

### Multi-threaded Mode

- **Queues:** Per-thread deques with work stealing (`crossbeam_deque`).
- **Per-thread arenas:** Sized for peak concurrent subtree per worker.
- **In-flight counter:** `AtomicUsize` to track active expansion tasks.
- **Leaf buffers:** Per-thread `Vec<Leaf>` merged at end.

---

## Concurrency Design

### Work Queues

- **Preferred:** `crossbeam_deque` for low-latency work stealing.
- **Alternative:** `crossbeam::queue::SegQueue` for simplicity.

### Per-thread State

```rust
pub struct Worker {
    pub id: WorkerId,
    pub arena: Arena,
    pub deque: crossbeam_deque::Worker<Task>,
    pub stealers: Vec<crossbeam_deque::Stealer<Task>>,
    pub leaves: Vec<Leaf>,
}
```

---

## Implementation Steps

### Phase 0: Prep
- Define `NodeId`, `WorkerId`, `Ply`, `Depth`, `TerminalKind`.
- Refactor `Arena` to use `NodeId`.

### Phase 1: Single-threaded Prototype
- Implement `Arena` with typed IDs.
- Implement sequential queue loop using references in `Task`.
- Validate correctness against current DFS.

### Phase 2: Harden Interfaces
- Hide raw indices; only `Arena` manipulates them.
- Add diagnostics: high-water mark, optional poison checks.

### Phase 3: Parallel Runtime
- Integrate `crossbeam_deque`.
- Create per-thread arenas and queues.
- Implement in-flight counter and termination detection.

### Phase 4: Validation and Profiling
- Perft-style tests for node counts.
- Measure nodes/sec, queue contention, arena usage.

### Phase 5: Engine Integration
- UCI protocol compliance.
- Stop flag for time controls.
- Best move selection from aggregated results.

---

## Testing and Benchmarking

- **Correctness:** Perft counts, terminal case parity.
- **Performance:** Nodes/sec scaling with threads.
- **Diagnostics:** Arena high-water mark, task counters.

---

## Future Extensions

- Move ordering before enqueue.
- Transposition-aware expansion.
- NUMA-aware arena allocation.
- Overflow handling if arena capacity exceeded.

---
```

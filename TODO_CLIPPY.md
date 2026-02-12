# TODO List: Clippy
Generated: 2026-02-12 14:24:25
**Stats**: 3 total | 0 not started | 0 in progress | 2 completed
---

## Completed

### [x] CLIP-001: clippy::module_inception: mod.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\mod.rs

warning: module has the same name as its containing module
 --> engine\src\search\mod.rs:1:1
  |
1 | pub mod search;
  | ^^^^^^^^^^^^^^^
  |
  = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#module_inception
  = note: `#[warn(clippy::module_inception)]` on by default



*Completed: 2026-02-12T06:10:18.495348*

### [x] CLIP-002: clippy::too_many_arguments: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: this function has too many arguments (11/7)
  --> engine\src\search\core.rs:78:1
   |
78 | / pub fn search_node_with_arena<M: MoveGenerator, E: Evaluator>(
79 | |     movegen: &M,
80 | |     evaluator: &E,
81 | |     arena: &mut Arena,
...  |
89 | |     start_time: Option<&std::time::Instant>,
90 | | ) -> i32 {
   | |________^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#too_many_arguments
   = note: `#[warn(clippy::too_many_arguments)]` on by default



*Completed: 2026-02-12T06:14:19.526772*

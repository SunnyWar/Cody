# TODO List: Clippy
Generated: 2026-02-14 16:24:28
**Stats**: 3 total | 1 not started | 1 in progress | 0 completed | 1 failed
---

## In Progress

### [ ] clippy-engine_src_search_core.rs-158-clippy_collapsible_if: clippy::collapsible_if: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: this `if` statement can be collapsed
   --> engine\src\search\core.rs:158:5
    |
158 | /     if let Some(e) = tt_exact_needs_verify {
159 | |         if !e.best_move.is_null() && moves_vec.contains(&e.best_move) {
160 | |             return e.value;
161 | |         }
162 | |     }
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
    = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
    |
158 ~     if let Some(e) = tt_exact_needs_verify
159 ~         && !e.best_move.is_null() && moves_vec.contains(&e.best_move) {
160 |             return e.value;
161 ~         }
    |



## Not Started

### [ ] clippy-engine_src_perft_integration_test.rs-130-clippy_collapsible_if: clippy::collapsible_if: perft_integration_test.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\perft_integration_test.rs

warning: this `if` statement can be collapsed
   --> engine\src\perft_integration_test.rs:130:13
    |
130 | /             if let Some(file) = dis_file {
131 | |                 if mv.from().file_char() != file {
132 | |                     continue;
133 | |                 }
134 | |             }
    | |_____________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
help: collapse nested if block
    |
130 ~             if let Some(file) = dis_file
131 ~                 && mv.from().file_char() != file {
132 |                     continue;
133 ~                 }
    |



## Failed

### [ ] clippy-engine_src_search_core.rs-78-clippy_too_many_arguments: clippy::too_many_arguments: core.rs
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



*Completed: 2026-02-14T14:03:57.385635*

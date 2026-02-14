# TODO List: Clippy
Generated: 2026-02-13 23:33:42
**Stats**: 54 total | 14 not started | 0 in progress | 5 completed | 25 failed
---

## Not Started

### [ ] CLIP-041: clippy::module_inception: mod.rs
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



### [ ] CLIP-042: clippy::too_many_arguments: core.rs
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



### [ ] CLIP-043: clippy::collapsible_if: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: this `if` statement can be collapsed
   --> engine\src\search\core.rs:155:5
    |
155 | /     if let Some(e) = tt_exact_needs_verify {
156 | |         if moves.iter().any(|mm| *mm == e.best_move) {
157 | |             return e.value;
158 | |         }
159 | |     }
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
    = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
    |
155 ~     if let Some(e) = tt_exact_needs_verify
156 ~         && moves.iter().any(|mm| *mm == e.best_move) {
157 |             return e.value;
158 ~         }
    |



### [ ] CLIP-044: clippy::manual_contains: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: using `contains()` instead of `iter().any()` is more efficient
   --> engine\src\search\core.rs:156:12
    |
156 |         if moves.iter().any(|mm| *mm == e.best_move) {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: try: `moves.contains(&e.best_move)`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#manual_contains
    = note: `#[warn(clippy::manual_contains)]` on by default



### [ ] CLIP-045: clippy::module_inception: mod.rs
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



### [ ] CLIP-046: clippy::too_many_arguments: core.rs
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



### [ ] CLIP-047: clippy::collapsible_if: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: this `if` statement can be collapsed
   --> engine\src\search\core.rs:155:5
    |
155 | /     if let Some(e) = tt_exact_needs_verify {
156 | |         if moves.iter().any(|mm| *mm == e.best_move) {
157 | |             return e.value;
158 | |         }
159 | |     }
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
    = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
    |
155 ~     if let Some(e) = tt_exact_needs_verify
156 ~         && moves.iter().any(|mm| *mm == e.best_move) {
157 |             return e.value;
158 ~         }
    |



### [ ] CLIP-048: clippy::manual_contains: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: using `contains()` instead of `iter().any()` is more efficient
   --> engine\src\search\core.rs:156:12
    |
156 |         if moves.iter().any(|mm| *mm == e.best_move) {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: try: `moves.contains(&e.best_move)`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#manual_contains
    = note: `#[warn(clippy::manual_contains)]` on by default



### [ ] CLIP-049: clippy::collapsible_if: perft_integration_test.rs
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



### [ ] CLIP-050: clippy::collapsible_if: perft_integration_test.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\perft_integration_test.rs

warning: this `if` statement can be collapsed
   --> engine\src\perft_integration_test.rs:135:13
    |
135 | /             if let Some(rank) = dis_rank {
136 | |                 if mv.from().rank_char() != rank {
137 | |                     continue;
138 | |                 }
139 | |             }
    | |_____________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
help: collapse nested if block
    |
135 ~             if let Some(rank) = dis_rank
136 ~                 && mv.from().rank_char() != rank {
137 |                     continue;
138 ~                 }
    |



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



### [ ] clippy-engine_src_search_core.rs-155-clippy_collapsible_if: clippy::collapsible_if: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: this `if` statement can be collapsed
   --> engine\src\search\core.rs:155:5
    |
155 | /     if let Some(e) = tt_exact_needs_verify {
156 | |         if moves.iter().any(|mm| *mm == e.best_move) {
157 | |             return e.value;
158 | |         }
159 | |     }
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
    = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
    |
155 ~     if let Some(e) = tt_exact_needs_verify
156 ~         && moves.iter().any(|mm| *mm == e.best_move) {
157 |             return e.value;
158 ~         }
    |



### [ ] clippy-engine_src_search_core.rs-156-clippy_manual_contains: clippy::manual_contains: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: using `contains()` instead of `iter().any()` is more efficient
   --> engine\src\search\core.rs:156:12
    |
156 |         if moves.iter().any(|mm| *mm == e.best_move) {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: try: `moves.contains(&e.best_move)`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#manual_contains
    = note: `#[warn(clippy::manual_contains)]` on by default



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



*Completed: 2026-02-12T22:26:28.582605*

### [x] CLIP-003: clippy::collapsible_if: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: this `if` statement can be collapsed
   --> engine\src\search\core.rs:155:5
    |
155 | /     if let Some(e) = tt_exact_needs_verify {
156 | |         if moves.iter().any(|mm| *mm == e.best_move) {
157 | |             return e.value;
158 | |         }
159 | |     }
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
    = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
    |
155 ~     if let Some(e) = tt_exact_needs_verify
156 ~         && moves.iter().any(|mm| *mm == e.best_move) {
157 |             return e.value;
158 ~         }
    |



*Completed: 2026-02-13T18:21:18.503316*

### [x] CLIP-004: clippy::manual_contains: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: using `contains()` instead of `iter().any()` is more efficient
   --> engine\src\search\core.rs:156:12
    |
156 |         if moves.iter().any(|mm| *mm == e.best_move) {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: try: `moves.contains(&e.best_move)`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#manual_contains
    = note: `#[warn(clippy::manual_contains)]` on by default



*Completed: 2026-02-13T18:28:52.031266*

### [x] CLIP-021: clippy::module_inception: mod.rs
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
  = note: `-W clippy::module-inception` implied by `-W clippy::style`
  = help: to override `-W clippy::style` add `#[allow(clippy::module_inception)]`



*Completed: 2026-02-13T21:01:03.962932*

### [x] CLIP-023: clippy::collapsible_if: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: this `if` statement can be collapsed
   --> engine\src\search\core.rs:155:5
    |
155 | /     if let Some(e) = tt_exact_needs_verify {
156 | |         if moves.iter().any(|mm| *mm == e.best_move) {
157 | |             return e.value;
158 | |         }
159 | |     }
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
    = note: `-W clippy::collapsible-if` implied by `-W clippy::style`
    = help: to override `-W clippy::style` add `#[allow(clippy::collapsible_if)]`
help: collapse nested if block
    |
155 ~     if let Some(e) = tt_exact_needs_verify
156 ~         && moves.iter().any(|mm| *mm == e.best_move) {
157 |             return e.value;
158 ~         }
    |



*Completed: 2026-02-13T21:19:40.808418*

## Failed

### [ ] CLIP-002: clippy::too_many_arguments: core.rs
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



*Completed: 2026-02-13T23:24:39.963038*

### [ ] CLIP-006: clippy::too_many_arguments: core.rs
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



*Completed: 2026-02-13T18:35:02.042662*

### [ ] CLIP-007: clippy::collapsible_if: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: this `if` statement can be collapsed
   --> engine\src\search\core.rs:155:5
    |
155 | /     if let Some(e) = tt_exact_needs_verify {
156 | |         if moves.iter().any(|mm| *mm == e.best_move) {
157 | |             return e.value;
158 | |         }
159 | |     }
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
    = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
    |
155 ~     if let Some(e) = tt_exact_needs_verify
156 ~         && moves.iter().any(|mm| *mm == e.best_move) {
157 |             return e.value;
158 ~         }
    |



*Completed: 2026-02-13T18:41:19.175901*

### [ ] CLIP-009: clippy::collapsible_if: perft_integration_test.rs
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



*Completed: 2026-02-13T18:56:49.322461*

### [ ] CLIP-010: clippy::collapsible_if: perft_integration_test.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\perft_integration_test.rs

warning: this `if` statement can be collapsed
   --> engine\src\perft_integration_test.rs:135:13
    |
135 | /             if let Some(rank) = dis_rank {
136 | |                 if mv.from().rank_char() != rank {
137 | |                     continue;
138 | |                 }
139 | |             }
    | |_____________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
help: collapse nested if block
    |
135 ~             if let Some(rank) = dis_rank
136 ~                 && mv.from().rank_char() != rank {
137 |                     continue;
138 ~                 }
    |



*Completed: 2026-02-13T19:04:07.816393*

### [ ] CLIP-012: clippy::too_many_arguments: core.rs
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



*Completed: 2026-02-13T19:11:18.371301*

### [ ] CLIP-013: clippy::collapsible_if: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: this `if` statement can be collapsed
   --> engine\src\search\core.rs:155:5
    |
155 | /     if let Some(e) = tt_exact_needs_verify {
156 | |         if moves.iter().any(|mm| *mm == e.best_move) {
157 | |             return e.value;
158 | |         }
159 | |     }
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
    = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
    |
155 ~     if let Some(e) = tt_exact_needs_verify
156 ~         && moves.iter().any(|mm| *mm == e.best_move) {
157 |             return e.value;
158 ~         }
    |



*Completed: 2026-02-13T19:16:46.016574*

### [ ] CLIP-016: clippy::too_many_arguments: core.rs
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



*Completed: 2026-02-13T20:21:06.942029*

### [ ] CLIP-017: clippy::collapsible_if: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: this `if` statement can be collapsed
   --> engine\src\search\core.rs:155:5
    |
155 | /     if let Some(e) = tt_exact_needs_verify {
156 | |         if moves.iter().any(|mm| *mm == e.best_move) {
157 | |             return e.value;
158 | |         }
159 | |     }
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
    = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
    |
155 ~     if let Some(e) = tt_exact_needs_verify
156 ~         && moves.iter().any(|mm| *mm == e.best_move) {
157 |             return e.value;
158 ~         }
    |



*Completed: 2026-02-13T20:28:05.053535*

### [ ] CLIP-019: clippy::collapsible_if: perft_integration_test.rs
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



*Completed: 2026-02-13T20:44:16.348079*

### [ ] CLIP-020: clippy::collapsible_if: perft_integration_test.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\perft_integration_test.rs

warning: this `if` statement can be collapsed
   --> engine\src\perft_integration_test.rs:135:13
    |
135 | /             if let Some(rank) = dis_rank {
136 | |                 if mv.from().rank_char() != rank {
137 | |                     continue;
138 | |                 }
139 | |             }
    | |_____________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
help: collapse nested if block
    |
135 ~             if let Some(rank) = dis_rank
136 ~                 && mv.from().rank_char() != rank {
137 |                     continue;
138 ~                 }
    |



*Completed: 2026-02-13T20:58:31.354587*

### [ ] CLIP-022: clippy::too_many_arguments: core.rs
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



*Completed: 2026-02-13T21:08:30.800234*

### [ ] CLIP-025: clippy::module_inception: mod.rs
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
  = note: `-W clippy::module-inception` implied by `-W clippy::style`
  = help: to override `-W clippy::style` add `#[allow(clippy::module_inception)]`



*Completed: 2026-02-13T21:43:59.025879*

### [ ] CLIP-026: clippy::too_many_arguments: core.rs
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



*Completed: 2026-02-13T21:49:16.178852*

### [ ] CLIP-027: clippy::collapsible_if: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: this `if` statement can be collapsed
   --> engine\src\search\core.rs:155:5
    |
155 | /     if let Some(e) = tt_exact_needs_verify {
156 | |         if moves.iter().any(|mm| *mm == e.best_move) {
157 | |             return e.value;
158 | |         }
159 | |     }
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
    = note: `-W clippy::collapsible-if` implied by `-W clippy::style`
    = help: to override `-W clippy::style` add `#[allow(clippy::collapsible_if)]`
help: collapse nested if block
    |
155 ~     if let Some(e) = tt_exact_needs_verify
156 ~         && moves.iter().any(|mm| *mm == e.best_move) {
157 |             return e.value;
158 ~         }
    |



*Completed: 2026-02-13T21:54:44.218913*

### [ ] CLIP-029: clippy::collapsible_if: perft_integration_test.rs
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



*Completed: 2026-02-13T22:08:57.099175*

### [ ] CLIP-030: clippy::collapsible_if: perft_integration_test.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\perft_integration_test.rs

warning: this `if` statement can be collapsed
   --> engine\src\perft_integration_test.rs:135:13
    |
135 | /             if let Some(rank) = dis_rank {
136 | |                 if mv.from().rank_char() != rank {
137 | |                     continue;
138 | |                 }
139 | |             }
    | |_____________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
help: collapse nested if block
    |
135 ~             if let Some(rank) = dis_rank
136 ~                 && mv.from().rank_char() != rank {
137 |                     continue;
138 ~                 }
    |



*Completed: 2026-02-13T22:17:51.166785*

### [ ] CLIP-031: clippy::module_inception: mod.rs
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



*Completed: 2026-02-13T22:18:59.195397*

### [ ] CLIP-032: clippy::too_many_arguments: core.rs
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
   = note: `-W clippy::too-many-arguments` implied by `-W clippy::complexity`
   = help: to override `-W clippy::complexity` add `#[allow(clippy::too_many_arguments)]`



*Completed: 2026-02-13T22:24:09.275011*

### [ ] CLIP-033: clippy::collapsible_if: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: this `if` statement can be collapsed
   --> engine\src\search\core.rs:155:5
    |
155 | /     if let Some(e) = tt_exact_needs_verify {
156 | |         if moves.iter().any(|mm| *mm == e.best_move) {
157 | |             return e.value;
158 | |         }
159 | |     }
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
    = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
    |
155 ~     if let Some(e) = tt_exact_needs_verify
156 ~         && moves.iter().any(|mm| *mm == e.best_move) {
157 |             return e.value;
158 ~         }
    |



*Completed: 2026-02-13T22:28:42.685068*

### [ ] CLIP-035: clippy::module_inception: mod.rs
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



*Completed: 2026-02-13T22:34:15.546804*

### [ ] CLIP-036: clippy::too_many_arguments: core.rs
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
   = note: `-W clippy::too-many-arguments` implied by `-W clippy::complexity`
   = help: to override `-W clippy::complexity` add `#[allow(clippy::too_many_arguments)]`



*Completed: 2026-02-13T22:39:18.443766*

### [ ] CLIP-037: clippy::collapsible_if: core.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\search\core.rs

warning: this `if` statement can be collapsed
   --> engine\src\search\core.rs:155:5
    |
155 | /     if let Some(e) = tt_exact_needs_verify {
156 | |         if moves.iter().any(|mm| *mm == e.best_move) {
157 | |             return e.value;
158 | |         }
159 | |     }
    | |_____^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
    = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
    |
155 ~     if let Some(e) = tt_exact_needs_verify
156 ~         && moves.iter().any(|mm| *mm == e.best_move) {
157 |             return e.value;
158 ~         }
    |



*Completed: 2026-02-13T22:44:02.731502*

### [ ] CLIP-039: clippy::collapsible_if: perft_integration_test.rs
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



*Completed: 2026-02-13T22:55:01.378387*

### [ ] CLIP-040: clippy::collapsible_if: perft_integration_test.rs
- **Priority**: medium
- **Category**: clippy
- **Complexity**: small
- **Files**: engine\src\perft_integration_test.rs

warning: this `if` statement can be collapsed
   --> engine\src\perft_integration_test.rs:135:13
    |
135 | /             if let Some(rank) = dis_rank {
136 | |                 if mv.from().rank_char() != rank {
137 | |                     continue;
138 | |                 }
139 | |             }
    | |_____________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
help: collapse nested if block
    |
135 ~             if let Some(rank) = dis_rank
136 ~                 && mv.from().rank_char() != rank {
137 |                     continue;
138 ~                 }
    |



*Completed: 2026-02-13T23:33:42.134687*

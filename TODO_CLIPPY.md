# TODO List: Clippy
Generated: 2026-02-12 22:26:28
**Stats**: 50 total | 48 not started | 1 in progress | 1 completed | 0 failed
---

## In Progress

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



## Not Started

### [ ] CLIP-003: clippy::collapsible_if: core.rs
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



### [ ] CLIP-004: clippy::manual_contains: core.rs
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



### [ ] CLIP-005: clippy::module_inception: mod.rs
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



### [ ] CLIP-008: clippy::manual_contains: core.rs
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



### [ ] CLIP-011: clippy::module_inception: mod.rs
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



### [ ] CLIP-014: clippy::manual_contains: core.rs
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
    = note: `-W clippy::manual-contains` implied by `-W clippy::perf`
    = help: to override `-W clippy::perf` add `#[allow(clippy::manual_contains)]`



### [ ] CLIP-015: clippy::module_inception: mod.rs
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



### [ ] CLIP-018: clippy::manual_contains: core.rs
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
    = note: `-W clippy::manual-contains` implied by `-W clippy::perf`
    = help: to override `-W clippy::perf` add `#[allow(clippy::manual_contains)]`



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



### [ ] CLIP-021: clippy::module_inception: mod.rs
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



### [ ] CLIP-023: clippy::collapsible_if: core.rs
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



### [ ] CLIP-024: clippy::manual_contains: core.rs
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



### [ ] CLIP-028: clippy::manual_contains: core.rs
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



### [ ] CLIP-034: clippy::manual_contains: core.rs
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



### [ ] CLIP-038: clippy::manual_contains: core.rs
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

use bitboard::mov::ChessMove;
use bitboard::movegen::generate_legal_moves_fast;
use bitboard::position::Position;
use once_cell::sync::Lazy;
use shakmaty::CastlingMode;
use shakmaty::Chess;
use shakmaty::fen::Fen;
use shakmaty_syzygy::AmbiguousWdl;
use shakmaty_syzygy::Tablebase;
use std::path::Path;
use std::sync::RwLock;

/// Very large score used for tablebase-proven wins/losses.
const TB_WIN_CP: i32 = 20_000;
/// Small score used for cursed/blessed outcomes in 50-move constrained endings.
const TB_CURSED_CP: i32 = 120;

static TABLEBASE: Lazy<RwLock<Option<Tablebase<Chess>>>> = Lazy::new(|| RwLock::new(None));

/// Configure Syzygy paths. Accepts one path or a ';'-separated list.
pub fn set_syzygy_path(path_list: &str) -> Result<(), String> {
    let trimmed = path_list.trim();
    if trimmed.is_empty() {
        *TABLEBASE.write().map_err(|_| "tablebase lock poisoned")? = None;
        return Ok(());
    }

    let mut tb = Tablebase::<Chess>::new();
    for dir in trimmed.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        tb.add_directory(Path::new(dir))
            .map_err(|e| format!("failed to add syzygy directory '{dir}': {e}"))?;
    }

    *TABLEBASE.write().map_err(|_| "tablebase lock poisoned")? = Some(tb);
    Ok(())
}

pub fn has_tablebases() -> bool {
    TABLEBASE.read().map(|g| g.is_some()).unwrap_or(false)
}

/// Probe WDL and return a score from side-to-move perspective.
pub fn probe_wdl_cp(pos: &Position) -> Option<i32> {
    if pos.all_pieces().count() > 7 {
        return None;
    }

    let fen_str = pos.to_fen();
    let fen: Fen = fen_str.parse().ok()?;
    let chess: Chess = fen.into_position(CastlingMode::Standard).ok()?;

    let guard = TABLEBASE.read().ok()?;
    let tb = guard.as_ref()?;
    let wdl = tb.probe_wdl(&chess).ok()?;

    let score = match wdl {
        AmbiguousWdl::Win | AmbiguousWdl::MaybeWin => TB_WIN_CP,
        AmbiguousWdl::CursedWin => TB_CURSED_CP,
        AmbiguousWdl::Draw => 0,
        AmbiguousWdl::BlessedLoss => -TB_CURSED_CP,
        AmbiguousWdl::Loss | AmbiguousWdl::MaybeLoss => -TB_WIN_CP,
    };

    Some(score)
}

/// Probe tablebases for all legal root moves and pick the best one.
///
/// Returns `None` when tablebases are not configured, the position is not
/// tablebase-eligible (>7 pieces), or probing is incomplete for any legal move.
pub fn probe_root_best_move(pos: &Position) -> Option<ChessMove> {
    if !has_tablebases() || pos.all_pieces().count() > 7 {
        return None;
    }

    let moves = generate_legal_moves_fast(pos);
    if moves.is_empty() {
        return None;
    }

    let mut best_move = ChessMove::null();
    let mut best_score = i32::MIN;

    for i in 0..moves.len() {
        let mv = moves[i];
        let mut child = Position::default();
        pos.apply_move_into(&mv, &mut child);

        // `probe_wdl_cp` is from side-to-move perspective in child, so negate.
        let root_score = -probe_wdl_cp(&child)?;
        if best_move.is_null() || root_score > best_score {
            best_score = root_score;
            best_move = mv;
        }
    }

    (!best_move.is_null()).then_some(best_move)
}

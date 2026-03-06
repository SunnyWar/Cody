#[derive(Debug, Clone, Copy)]
pub struct GoLimits {
    pub depth: Option<usize>,
    pub movetime_ms: Option<u64>,
    pub wtime_ms: Option<u64>,
    pub btime_ms: Option<u64>,
    pub winc_ms: Option<u64>,
    pub binc_ms: Option<u64>,
    // flags
    pub ponder: bool,
    pub infinite: bool,
}

impl Default for GoLimits {
    fn default() -> Self {
        GoLimits {
            depth: None,
            movetime_ms: None,
            wtime_ms: None,
            btime_ms: None,
            winc_ms: None,
            binc_ms: None,
            ponder: false,
            infinite: false,
        }
    }
}

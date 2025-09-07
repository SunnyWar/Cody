/// Compact move. You’ll replace fields with your bitpacking of choice.
#[derive(Copy, Clone, Default)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub promo: u8, // 0=none, 1=Q,2=R,3=B,4=N
}


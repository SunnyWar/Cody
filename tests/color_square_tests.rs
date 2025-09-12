use cody::generated::SQUARE_COLOR_MASK;

#[test]
fn test_square_color_mask_values() {
    const BOARD_SIZE: usize = 8;
    const NUM_SQUARES: usize = BOARD_SIZE * BOARD_SIZE;

    // Precompute expected values
    let mut expected = [0u64; NUM_SQUARES];
    let mut light_squares: u64 = 0;

    for sq in 0..NUM_SQUARES {
        let rank = sq / BOARD_SIZE;
        let file = sq % BOARD_SIZE;
        if (rank + file).is_multiple_of(2) {
            light_squares |= 1u64 << sq;
        }
    }

    let dark_squares = !light_squares;

    for sq in 0..NUM_SQUARES {
        let rank = sq / BOARD_SIZE;
        let file = sq % BOARD_SIZE;
        expected[sq] = if (rank + file).is_multiple_of(2) {
            light_squares
        } else {
            dark_squares
        };
    }

    // Compare each entry
    for sq in 0..NUM_SQUARES {
        assert_eq!(
            SQUARE_COLOR_MASK[sq], expected[sq],
            "Mismatch at square {}: expected 0x{:016X}, got 0x{:016X}",
            sq, expected[sq], SQUARE_COLOR_MASK[sq]
        );
    }
}

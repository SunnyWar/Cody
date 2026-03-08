// Test to verify alignment of performance-critical structures
use std::mem::align_of;
use std::mem::size_of;

#[test]
fn test_structure_alignment() {
    use bitboard::intrinsics::SimdI32x8;
    use bitboard::intrinsics::SimdU64x4;
    use bitboard::movelist::MoveList;
    use bitboard::piecebitboards::PieceBitboards;
    use bitboard::position::MoveGenContext;
    use bitboard::position::Position;
    use engine::core::node::Node;
    use engine::core::tt::TTEntry;

    println!("\n=== PERFORMANCE-CRITICAL STRUCTURE ALIGNMENT ANALYSIS ===\n");

    println!("BITBOARD STRUCTURES:");
    print_info::<Position>("Position");
    print_info::<PieceBitboards>("PieceBitboards");
    print_info::<MoveGenContext>("MoveGenContext");
    print_info::<MoveList>("MoveList");

    println!("\nSIMD STRUCTURES:");
    print_info::<SimdU64x4>("SimdU64x4");
    print_info::<SimdI32x8>("SimdI32x8");

    println!("\nENGINE STRUCTURES:");
    print_info::<Node>("Node");
    print_info::<TTEntry>("TTEntry");

    println!("\n=== CACHE LINE ANALYSIS (64-byte cache lines) ===");
    cache_analysis::<Position>("Position");
    cache_analysis::<TTEntry>("TTEntry");
    cache_analysis::<Node>("Node");
    cache_analysis::<MoveList>("MoveList");
    cache_analysis::<PieceBitboards>("PieceBitboards");

    println!("\n=== ALIGNMENT RECOMMENDATIONS ===");
    // Check if alignment is sufficient for hot path structures
    check_alignment::<Position>("Position", 64, true);
    check_alignment::<TTEntry>("TTEntry", 64, true);
    check_alignment::<PieceBitboards>("PieceBitboards", 64, true);
    check_alignment::<MoveList>("MoveList", 64, false); // Already aligned
    check_alignment::<SimdU64x4>("SimdU64x4", 32, false); // Already aligned
    check_alignment::<SimdI32x8>("SimdI32x8", 32, false); // Already aligned
}

fn print_info<T>(name: &str) {
    let size = size_of::<T>();
    let align = align_of::<T>();
    let status = if align >= 64 {
        "✓ CACHE LINE"
    } else if align >= 32 {
        "✓ AVX2"
    } else if align >= 16 {
        "⚠ SSE"
    } else if align >= 8 {
        "⚠ NATURAL"
    } else {
        "✗ POOR"
    };

    println!(
        "  {:<20}: size={:4} bytes, align={:2} bytes  [{}]",
        name, size, align, status
    );
}

fn cache_analysis<T>(name: &str) {
    let size = size_of::<T>();
    let align = align_of::<T>();
    let cache_lines = size.div_ceil(64);
    let waste = cache_lines * 64 - size;
    let efficiency = 100.0 * (size as f64) / (cache_lines as f64 * 64.0);

    println!(
        "  {:<20}: {} cache line(s), {} bytes waste ({:.1}% efficient), align={}",
        name, cache_lines, waste, efficiency, align
    );
}

fn check_alignment<T>(name: &str, target: usize, suggest: bool) {
    let current = align_of::<T>();
    let size = size_of::<T>();

    if current < target && suggest {
        // For large structures accessed frequently, cache line alignment is beneficial
        if size >= 32 {
            println!(
                "  ⚠️  {:<20}: SHOULD add #[repr(align({}))] - accessed frequently in hot path",
                name, target
            );
        }
    } else if current >= target {
        println!("  ✓  {:<20}: Already properly aligned ({})", name, current);
    }
}

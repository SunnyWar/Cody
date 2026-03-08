// Temporary script to check alignment of performance-critical structures
use std::mem::align_of;
use std::mem::size_of;

fn main() {
    println!("=== PERFORMANCE-CRITICAL STRUCTURE ALIGNMENT ANALYSIS ===\n");

    // Import types
    use bitboard::intrinsics::SimdI32x8;
    use bitboard::intrinsics::SimdU64x4;
    use bitboard::movelist::MoveList;
    use bitboard::piecebitboards::PieceBitboards;
    use bitboard::position::MoveGenContext;
    use bitboard::position::Position;
    use engine::core::node::Node;
    use engine::core::tt::TTEntry;

    println!("BITBOARD STRUCTURES:");
    println!(
        "  Position          : size={:4} bytes, align={:2} bytes{}",
        size_of::<Position>(),
        align_of::<Position>(),
        check_alignment::<Position>()
    );
    println!(
        "  PieceBitboards    : size={:4} bytes, align={:2} bytes{}",
        size_of::<PieceBitboards>(),
        align_of::<PieceBitboards>(),
        check_alignment::<PieceBitboards>()
    );
    println!(
        "  MoveGenContext    : size={:4} bytes, align={:2} bytes{}",
        size_of::<MoveGenContext>(),
        align_of::<MoveGenContext>(),
        check_alignment::<MoveGenContext>()
    );
    println!(
        "  MoveList          : size={:4} bytes, align={:2} bytes{}",
        size_of::<MoveList>(),
        align_of::<MoveList>(),
        check_alignment::<MoveList>()
    );

    println!("\nSIMD STRUCTURES:");
    println!(
        "  SimdU64x4         : size={:4} bytes, align={:2} bytes{}",
        size_of::<SimdU64x4>(),
        align_of::<SimdU64x4>(),
        check_alignment::<SimdU64x4>()
    );
    println!(
        "  SimdI32x8         : size={:4} bytes, align={:2} bytes{}",
        size_of::<SimdI32x8>(),
        align_of::<SimdI32x8>(),
        check_alignment::<SimdI32x8>()
    );

    println!("\nENGINE STRUCTURES:");
    println!(
        "  Node              : size={:4} bytes, align={:2} bytes{}",
        size_of::<Node>(),
        align_of::<Node>(),
        check_alignment::<Node>()
    );
    println!(
        "  TTEntry           : size={:4} bytes, align={:2} bytes{}",
        size_of::<TTEntry>(),
        align_of::<TTEntry>(),
        check_alignment::<TTEntry>()
    );

    println!("\n=== CACHE LINE ANALYSIS (64-byte cache lines) ===");
    print_cache_efficiency::<Position>("Position");
    print_cache_efficiency::<TTEntry>("TTEntry");
    print_cache_efficiency::<Node>("Node");
    print_cache_efficiency::<MoveList>("MoveList");

    println!("\n=== RECOMMENDATIONS ===");
    recommend_alignment::<Position>("Position", 64);
    recommend_alignment::<TTEntry>("TTEntry", 64);
    recommend_alignment::<PieceBitboards>("PieceBitboards", 64);
    recommend_alignment::<Node>("Node", 64);
}

fn check_alignment<T>() -> &'static str {
    let align = std::mem::align_of::<T>();
    if align >= 64 {
        " ✓ CACHE LINE ALIGNED"
    } else if align >= 32 {
        " ✓ AVX2 ALIGNED"
    } else if align >= 16 {
        " ⚠ SSE ALIGNED"
    } else if align >= 8 {
        " ⚠ NATURAL ALIGNMENT"
    } else {
        " ✗ POOR ALIGNMENT"
    }
}

fn print_cache_efficiency<T>(name: &str) {
    let size = std::mem::size_of::<T>();
    let align = std::mem::align_of::<T>();
    let cache_lines = (size + 63) / 64;
    let waste = cache_lines * 64 - size;

    println!(
        "  {:<20}: {} cache line(s), {} bytes waste, align={}",
        name, cache_lines, waste, align
    );
}

fn recommend_alignment<T>(name: &str, target: usize) {
    let current = std::mem::align_of::<T>();
    let size = std::mem::size_of::<T>();

    if current < target {
        if size >= target / 2 {
            println!(
                "  ⚠️  {:<20}: Consider adding #[repr(align({}))]",
                name, target
            );
        }
    }
}

use bitboard::BitBoardMask;
use bitboard::bitboard::occupancy_to_index;
use bitboard::movegen::SimpleMoveGen;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use engine::{Engine, MaterialEvaluator, NODE_COUNT, TEST_CASES};
use std::hint::black_box;
use std::sync::atomic::Ordering;

fn bench_search(c: &mut Criterion) {
    let depth = 3;
    let mut engine = Engine::new(65536, SimpleMoveGen, MaterialEvaluator);
    let mut group = c.benchmark_group("SearchBench");

    let mut cases: Vec<&engine::TestCase> = TEST_CASES.iter().collect();
    cases.sort_by(|a, b| a.name.cmp(b.name));

    for case in cases {
        group.bench_with_input(BenchmarkId::new("Position", case.name), case, |b, tc| {
            b.iter(|| {
                NODE_COUNT.store(0, Ordering::Relaxed);
                let _score = engine.search(black_box(&tc.position()), depth, None, None);
                black_box(NODE_COUNT.load(Ordering::Relaxed))
            });
        });
    }

    group.finish();
}

fn bench_occupancy_to_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("OccupancyIndex");

    let mask = BitBoardMask(0b10110);
    let occupancy = BitBoardMask(0b10010);

    group.bench_function("basic occupancy_to_index", |b| {
        b.iter(|| {
            let _ = black_box(occupancy_to_index(occupancy, mask));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_search, bench_occupancy_to_index);
criterion_main!(benches);

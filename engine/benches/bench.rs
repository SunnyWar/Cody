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
                let _score = engine.search(black_box(&tc.position()), depth);
                black_box(NODE_COUNT.load(Ordering::Relaxed))
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_search);
criterion_main!(benches);

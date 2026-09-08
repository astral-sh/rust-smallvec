use {
    criterion::{
        BatchSize,
        BenchmarkId,
        Criterion,
        criterion_group,
        criterion_main
    },
    smallvec::SmallVec,
    std::{
        hint::black_box,
        time::Duration
    }
};
fn benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("dedup_compaction");
    for n in [16, 17, 4096] {
        for (pattern, mode) in [("unique", 0), ("pairs", 1), ("equal", 2)] {
            let input: Vec<_> = (0..n)
                .map(|i| {
                    [match mode {
                        0 => i,
                        1 => i / 2,
                        _ => 0
                    } as u64; 16]
                })
                .collect();
            group.bench_with_input(BenchmarkId::new(pattern, n), &input, |b, input| {
                b.iter_batched_ref(
                    || SmallVec::<[u64; 16], 16>::from_slice_copy(input),
                    |v| {
                        v.dedup_by(|a, b| a[0] == b[0]);
                        black_box(v.as_slice());
                    },
                    BatchSize::LargeInput
                );
            });
        }
    }
    let input: Vec<_> = (0..4096).map(|i| (i / 2) as u64).collect();
    group.bench_function("u64_pairs_4096", |b| {
        b.iter_batched_ref(
            || SmallVec::<u64, 16>::from_slice_copy(&input),
            |v| {
                v.dedup();
                black_box(v.as_slice());
            },
            BatchSize::LargeInput
        );
    });
    let input: Vec<_> = (0..4096).map(|i| [(i / 2) as u64; 16]).collect();
    group.bench_function("vec_pairs_4096_control", |b| {
        b.iter_batched_ref(
            || input.clone(),
            |v| {
                v.dedup_by(|a, b| a[0] == b[0]);
                black_box(v.as_slice());
            },
            BatchSize::LargeInput
        );
    });
    group.finish();
}
criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2));
    targets = benchmarks
);
criterion_main!(benches);

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

fn collect(c: &mut Criterion) {
    let mut group = c.benchmark_group("collect");
    for len in [0, 7, 8, 9, 64, 1024] {
        group.bench_with_input(BenchmarkId::new("untouched", len), &len, |b, &len| {
            b.iter_batched(
                || (0..len).collect::<SmallVec<usize, 8>>().into_iter(),
                |iter| black_box(black_box(iter).collect::<SmallVec<usize, 8>>()),
                BatchSize::LargeInput
            );
        });
    }
    for len in [8, 64, 1024] {
        group.bench_with_input(BenchmarkId::new("partial", len), &len, |b, &len| {
            b.iter_batched(
                || {
                    let mut iter = (0..len).collect::<SmallVec<usize, 8>>().into_iter();
                    iter.next();
                    iter.next_back();
                    iter
                },
                |iter| black_box(black_box(iter).collect::<SmallVec<usize, 8>>()),
                BatchSize::LargeInput
            );
        });
    }
    group.bench_function("inline_to_heap/16", |b| {
        b.iter_batched(
            || (0..16).collect::<SmallVec<usize, 16>>().into_iter(),
            |iter| black_box(black_box(iter).collect::<SmallVec<usize, 8>>()),
            BatchSize::LargeInput
        );
    });
    group.bench_function("heap_to_inline/16", |b| {
        b.iter_batched(
            || (0..16).collect::<SmallVec<usize, 8>>().into_iter(),
            |iter| black_box(black_box(iter).collect::<SmallVec<usize, 16>>()),
            BatchSize::LargeInput
        );
    });
    group.bench_function("zst/64", |b| {
        b.iter_batched(
            || {
                std::iter::repeat_n((), 64)
                    .collect::<SmallVec<(), 8>>()
                    .into_iter()
            },
            |iter| black_box(black_box(iter).collect::<SmallVec<(), 8>>()),
            BatchSize::LargeInput
        );
    });
    group.bench_function("string/64", |b| {
        b.iter_batched(
            || {
                (0..64)
                    .map(|n| n.to_string())
                    .collect::<SmallVec<String, 8>>()
                    .into_iter()
            },
            |iter| black_box(black_box(iter).collect::<SmallVec<String, 8>>()),
            BatchSize::LargeInput
        );
    });
    group.bench_function("vec_control/64", |b| {
        b.iter_batched(
            || (0..64).collect::<Vec<usize>>().into_iter(),
            |iter| black_box(black_box(iter).collect::<Vec<usize>>()),
            BatchSize::LargeInput
        );
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2));
    targets = collect
}
criterion_main!(benches);

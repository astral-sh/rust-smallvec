use {
    criterion::{
        BatchSize,
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

#[derive(Clone)]
struct Dropped(u64);

impl Drop for Dropped {
    fn drop(&mut self) {
        black_box(self.0);
    }
}

fn case<T: Clone>(c: &mut Criterion, name: &str, len: usize, value: T) {
    let source: SmallVec<T, 16> = (0..len).map(|_| value.clone()).collect();
    let mut group = c.benchmark_group(name);
    group.bench_function("nth", |b| {
        b.iter_batched(
            || source.clone().into_iter(),
            |mut iter| black_box(iter.nth(black_box(len.saturating_sub(1)))),
            BatchSize::LargeInput
        );
    });
    group.bench_function("nth_back", |b| {
        b.iter_batched(
            || source.clone().into_iter(),
            |mut iter| black_box(iter.nth_back(black_box(len.saturating_sub(1)))),
            BatchSize::LargeInput
        );
    });
    group.bench_function("last", |b| {
        b.iter_batched(
            || source.clone().into_iter(),
            |iter| black_box(iter.last()),
            BatchSize::LargeInput
        );
    });
    // Control for moving/dropping the iterator without skipping elements.
    group.bench_function("next", |b| {
        b.iter_batched(
            || source.clone().into_iter(),
            |mut iter| black_box(iter.next()),
            BatchSize::LargeInput
        );
    });
    let source: Vec<T> = source.into_vec();
    group.bench_function("vec_nth", |b| {
        b.iter_batched(
            || source.clone().into_iter(),
            |mut iter| black_box(iter.nth(black_box(len.saturating_sub(1)))),
            BatchSize::LargeInput
        );
    });
    group.finish();
}

fn benches(c: &mut Criterion) {
    case(c, "empty", 0, 7u64);
    case(c, "inline", 8, 7u64);
    case(c, "boundary", 16, 7u64);
    case(c, "first_spill", 17, 7u64);
    case(c, "spilled", 4096, 7u64);
    case(c, "large_items", 256, [7u64; 16]);
    case(c, "drop_items", 256, Dropped(7));
    case(c, "zero_sized", 4096, ());
}

criterion_group! {
    name = benchmarks;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2));
    targets = benches
}
criterion_main!(benchmarks);

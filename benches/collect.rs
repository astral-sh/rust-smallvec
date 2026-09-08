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

struct Dropped(usize);

impl Drop for Dropped {
    fn drop(&mut self) {
        black_box(self.0);
    }
}

fn retention_case<T>(
    c: &mut Criterion,
    name: &str,
    capacity: usize,
    len: usize,
    front: usize,
    back: usize,
    item: impl Fn(usize) -> T
) {
    c.bench_function(name, |b| {
        b.iter_batched(
            || {
                let mut source = SmallVec::<T, 8>::with_capacity(capacity);
                source.extend((0..len).map(&item));
                let mut iter = source.into_iter();
                for _ in 0..front {
                    drop(iter.next());
                }
                for _ in 0..back {
                    drop(iter.next_back());
                }
                iter
            },
            |iter| black_box(black_box(iter).collect::<SmallVec<T, 8>>()),
            BatchSize::LargeInput
        );
    });
}

fn retention(c: &mut Criterion) {
    for (name, capacity, len, front, back) in [
        ("front", 4096, 4096, 4032, 0),
        ("back", 4096, 4096, 0, 4032),
        ("slack", 4096, 64, 0, 0),
        ("half", 128, 128, 64, 0),
        ("odd_below_half", 129, 129, 65, 0),
        ("odd_above_half", 129, 129, 64, 0)
    ] {
        retention_case(
            c,
            &format!("retention/{name}"),
            capacity,
            len,
            front,
            back,
            |n| n
        );
    }
    retention_case(c, "retention/drop_front", 256, 256, 240, 0, Dropped);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2));
    targets = collect, retention
}
criterion_main!(benches);

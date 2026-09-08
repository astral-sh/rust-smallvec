use {
    criterion::{
        BenchmarkId,
        Criterion,
        criterion_group,
        criterion_main
    },
    smallvec::from_elem,
    std::{
        hint::black_box,
        time::Duration
    }
};

#[allow(clippy::unit_arg)]
fn fill(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_elem");
    for n in [7, 8, 9, 64, 1024] {
        for value in [0u8, 19] {
            group.bench_with_input(BenchmarkId::new(format!("u8_{value}"), n), &n, |b, &n| {
                b.iter(|| black_box(from_elem::<_, 8>(black_box(value), black_box(n))));
            });
        }
    }
    group.bench_function("u64_19/64", |b| {
        b.iter(|| black_box(from_elem::<_, 8>(black_box(19u64), black_box(64))));
    });
    group.bench_function("u64_0/64", |b| {
        b.iter(|| black_box(from_elem::<_, 8>(black_box(0u64), black_box(64))));
    });
    group.bench_function("bool_false/64", |b| {
        b.iter(|| black_box(from_elem::<_, 8>(black_box(false), black_box(64))));
    });
    group.bench_function("string/64", |b| {
        b.iter(|| {
            black_box(from_elem::<_, 8>(
                black_box(String::from("smallvec")),
                black_box(64)
            ))
        });
    });
    group.bench_function("zst/64", |b| {
        b.iter(|| black_box(from_elem::<_, 8>(black_box(()), black_box(64))));
    });
    group.bench_function("vec_19_control/64", |b| {
        b.iter(|| black_box(vec![black_box(19u8); black_box(64)]));
    });
    group.bench_function("vec_0_control/1024", |b| {
        b.iter(|| black_box(vec![black_box(0u8); black_box(1024)]));
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2));
    targets = fill
}
criterion_main!(benches);

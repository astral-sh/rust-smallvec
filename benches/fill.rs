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
fn repeated_elements(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_elem");
    for n in [7, 8, 9, 64, 1024] {
        for value in [0u8, 19] {
            group.bench_with_input(BenchmarkId::new(format!("u8_{value}"), n), &n, |b, &n| {
                b.iter(|| black_box(from_elem::<_, 8>(black_box(value), black_box(n))));
            });
        }
    }
    group.bench_function("u64/64", |b| {
        b.iter(|| black_box(from_elem::<_, 8>(black_box(19u64), black_box(64))));
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
    group.bench_function("vec_control/64", |b| {
        b.iter(|| black_box(vec![black_box(19u8); black_box(64)]));
    });
    group.finish();
}

#[cfg(feature = "bytes")]
fn repeated_bytes(c: &mut Criterion) {
    use {
        bytes::BufMut,
        smallvec::SmallVec
    };

    let mut group = c.benchmark_group("put_bytes");
    for count in [0, 4, 5, 6, 64, 1024] {
        let mut buf = SmallVec::<u8, 8>::with_capacity(3 + count);
        buf.extend_from_slice_copy(&[1, 2, 3]);
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                buf.truncate(3);
                black_box(&mut buf).put_bytes(black_box(19), black_box(count));
                black_box(&buf);
            });
        });
    }
    group.finish();
}

fn fill(c: &mut Criterion) {
    repeated_elements(c);
    #[cfg(feature = "bytes")]
    repeated_bytes(c);
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

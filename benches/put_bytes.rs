use {
    bytes::BufMut,
    criterion::{
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

fn put_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("put_bytes");
    for count in [0, 4, 5, 6, 64, 1024] {
        let mut buf = SmallVec::<u8, 8>::with_capacity(3 + count);
        buf.extend_from_slice_copy(&[1, 2, 3]);
        group.bench_with_input(BenchmarkId::new("smallvec", count), &count, |b, &count| {
            b.iter(|| {
                buf.truncate(3);
                black_box(&mut buf).put_bytes(black_box(19), black_box(count));
                black_box(&buf);
            });
        });
    }
    for count in [0, 64, 1024] {
        let mut buf = Vec::with_capacity(3 + count);
        buf.extend_from_slice(&[1u8, 2, 3]);
        group.bench_with_input(
            BenchmarkId::new("vec_control", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    buf.truncate(3);
                    let count = black_box(count);
                    let buf = black_box(&mut buf);
                    buf.resize(buf.len() + count, black_box(19));
                    black_box(&buf);
                });
            }
        );
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2));
    targets = put_bytes
}
criterion_main!(benches);

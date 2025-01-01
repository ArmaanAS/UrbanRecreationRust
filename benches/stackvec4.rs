use criterion::{black_box, criterion_group, criterion_main, Criterion};
use urban_recreation_rust::utils::StackVec4;

fn push_benchmark(c: &mut Criterion) {
    c.bench_function("stackvec4_push", |b| {
        b.iter(|| {
            let mut vec = StackVec4::<u8>::default();
            vec.push(black_box(1));
            vec.push(black_box(2));
            vec.push(black_box(3));
            vec.push(black_box(4));
        })
    });

    c.bench_function("stackvec4_push_1", |b| {
        b.iter(|| {
            let mut vec = StackVec4::<u8>::default();
            vec.push_1(black_box(1));
            vec.push_1(black_box(2));
            vec.push_1(black_box(3));
            vec.push_1(black_box(4));
        })
    });

    c.bench_function("stackvec4_push_1_safe", |b| {
        b.iter(|| {
            let mut vec = StackVec4::<u8>::default();
            vec.push_1_safe(black_box(1));
            vec.push_1_safe(black_box(2));
            vec.push_1_safe(black_box(3));
            vec.push_1_safe(black_box(4));
        })
    });
}

criterion_group!(benches, push_benchmark);
criterion_main!(benches);

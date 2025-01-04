use adventofcode as aoc;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

const CASES: &[u64] = &[1, 10, 22, 100, 333, 1000, 4444, 10000, 55555];

pub fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("CountDigits");
    for i in CASES {
        group.bench_with_input(BenchmarkId::new("Log", *i), i, |b, i| {
            b.iter(|| black_box(aoc::_count_digits_with_log(black_box(*i))))
        });
        group.bench_with_input(BenchmarkId::new("Fast", *i), i, |b, i| {
            b.iter(|| black_box(aoc::_count_digits_fast(black_box(*i))))
        });
    }
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

#[path = "sum_str_digits/data.rs"]
mod data;

fn sum_with_unreachable(input: &str) -> u32 {
    input
        .bytes()
        .map(|b| match b {
            b'0'..=b'9' => (b - b'0') as u32,
            _ => unreachable!(),
        })
        .sum::<u32>()
}

fn sum_with_unsafe_unchecked(input: &str) -> u32 {
    input
        .bytes()
        .map(|b| match b {
            b'0'..=b'9' => (b - b'0') as u32,
            _ => unsafe { std::hint::unreachable_unchecked() },
        })
        .sum::<u32>()
}

pub fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("SumStrDigits");
    group.bench_with_input(
        BenchmarkId::new("Unreachable", "data"),
        data::DATA,
        |b, input| b.iter(|| black_box(sum_with_unreachable(black_box(input)))),
    );
    group.bench_with_input(
        BenchmarkId::new("Unchecked", "data"),
        data::DATA,
        |b, input| b.iter(|| black_box(sum_with_unsafe_unchecked(black_box(input)))),
    );
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

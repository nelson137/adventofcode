use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

const PAGE_SIZE: usize = 512; // 16384
const PAGE_MASK: usize = !(PAGE_SIZE - 1);

fn round_to_nearest_branch(x: usize) -> usize {
    let aligned = x & PAGE_MASK;
    if aligned == x {
        aligned
    } else {
        aligned + PAGE_SIZE
    }
}

fn round_to_nearest_nobranch(x: usize) -> usize {
    let aligned = x & PAGE_MASK;
    aligned + (x > aligned) as usize * PAGE_SIZE
}

pub fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("RoundToNearestPow2");

    #[allow(clippy::unusual_byte_groupings)]
    const TEST: usize = 0b_001000_0_000000001;

    group.bench_with_input(BenchmarkId::new("Branch", "_"), &TEST, |b, input| {
        b.iter(|| black_box(round_to_nearest_branch(black_box(*input))))
    });

    group.bench_with_input(BenchmarkId::new("NoBranch", "_"), &TEST, |b, input| {
        b.iter(|| black_box(round_to_nearest_nobranch(black_box(*input))))
    });
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

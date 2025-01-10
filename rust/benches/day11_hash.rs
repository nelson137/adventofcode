use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, DefaultHasher, Hasher},
};

use adventofcode as aoc;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

#[path = "day11_hash/data.rs"]
mod data;

fn try_split_digits(value: u64) -> Option<(u64, u64)> {
    let count = aoc::count_digits(value);
    if count % 2 == 0 {
        let factor = 10_u64.pow(count as u32 / 2);
        let l = value / factor;
        let r = value - l * factor;
        Some((l, r))
    } else {
        None
    }
}

fn solve<H: Hasher + Default>(input: &str) -> u64 {
    type Map<H> = HashMap<u64, u64, BuildHasherDefault<H>>;

    let mut stones = Map::<H>::default();
    for value in input.trim().split(" ").map(|r| r.parse::<u64>().unwrap()) {
        stones.insert(value, 1);
    }

    let mut next_stones = Map::<H>::default();

    for _ in 0..75 {
        for (value, count) in stones.drain() {
            if value == 0 {
                *next_stones.entry(1).or_default() += count;
            } else if let Some((l, r)) = try_split_digits(value) {
                *next_stones.entry(l).or_default() += count;
                *next_stones.entry(r).or_default() += count;
            } else {
                *next_stones.entry(value * 2024).or_default() += count;
            }
        }

        std::mem::swap(&mut stones, &mut next_stones);
    }

    stones.values().copied().sum::<u64>()
}

pub fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("Day11Hash");

    group.bench_with_input(BenchmarkId::new("Default", "_"), data::DATA, |b, input| {
        b.iter(|| black_box(solve::<DefaultHasher>(black_box(input))))
    });

    group.bench_with_input(BenchmarkId::new("Fx", "_"), data::DATA, |b, input| {
        b.iter(|| black_box(solve::<fxhash::FxHasher>(black_box(input))))
    });

    group.bench_with_input(BenchmarkId::new("Fx64", "_"), data::DATA, |b, input| {
        b.iter(|| black_box(solve::<fxhash::FxHasher64>(black_box(input))))
    });

    group.bench_with_input(
        BenchmarkId::new("Murmur3Mix", "_"),
        data::DATA,
        |b, input| b.iter(|| black_box(solve::<aoc::Murmur3MixHash64>(black_box(input)))),
    );
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

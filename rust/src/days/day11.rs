use std::collections::HashMap;

use adventofcode as aoc;
use rbtree::RBTree;

crate::day_executors! {
    [part1]
    [part2]
}

crate::day_visualizers! {
    []
    []
}

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

pub(super) fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut stones = input
        .trim()
        .split(" ")
        .map(|r| r.parse::<u64>().unwrap())
        .enumerate()
        .collect::<RBTree<_, _>>();

    let mut next_id = stones.len();
    let mut split_stones_right_side = Vec::<u64>::new();

    const N_BLINKS: u32 = 25;

    for _ in 0..N_BLINKS {
        for value in stones.values_mut() {
            if *value == 0 {
                *value = 1;
            } else if let Some((l, r)) = try_split_digits(*value) {
                *value = l;
                split_stones_right_side.push(r);
            } else {
                *value *= 2024;
            }
        }

        for r in split_stones_right_side.drain(..) {
            stones.insert(next_id, r);
            next_id += 1;
        }
    }

    let answer = stones.len();

    Some(Box::new(answer))
}

pub(super) fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut stones = input
        .trim()
        .split(" ")
        .map(|r| (r.parse::<u64>().unwrap(), 1_u64))
        .collect::<HashMap<_, _>>();
    let mut next_stones = HashMap::new();

    const N_BLINKS: u32 = 75;

    for _ in 0..N_BLINKS {
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

    let n_stones = stones.values().copied().sum::<u64>();

    Some(Box::new(n_stones))
}

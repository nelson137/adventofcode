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
    const N_BLINKS: u32 = 75;

    let mut stones = input
        .trim()
        .split(" ")
        .map(|r| (r.parse::<u64>().unwrap(), N_BLINKS))
        .collect::<Vec<_>>();

    let mut n_stones = 0_u64;

    while let Some((mut value, blinks_left)) = stones.pop() {
        n_stones += 1;
        for b in 1..=blinks_left {
            if value == 0 {
                value = 1;
            } else if let Some((l, r)) = try_split_digits(value) {
                value = l;
                stones.push((r, blinks_left - b));
            } else {
                value *= 2024;
            }
        }
    }

    Some(Box::new(n_stones))
}

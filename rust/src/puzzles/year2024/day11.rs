use adventofcode as aoc;

inventory::submit!(crate::days::DayModule::new(2024, 11).with_executors(
    crate::day_part_executors![part1],
    crate::day_part_executors![part2],
));

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

fn blink_in_infinite_corridor<const N_BLINKS: u32>(input: &str) -> u64 {
    let mut stones = aoc::HashMap::default();
    for value in input.trim().split(" ").map(|r| r.parse::<u64>().unwrap()) {
        stones.insert(value, 1);
    }

    let mut next_stones = aoc::HashMap::default();

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

    stones.values().copied().sum::<u64>()
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let n_stones = blink_in_infinite_corridor::<25>(input);
    Some(Box::new(n_stones))
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let n_stones = blink_in_infinite_corridor::<75>(input);
    Some(Box::new(n_stones))
}

use std::collections::HashMap;

crate::day_executors! {
    [part1]
    [part2]
}

crate::day_visualizers! {
    []
    []
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut left = Vec::<u32>::new();
    let mut right = Vec::<u32>::new();

    for line in input.lines() {
        let l = line[0..5].parse::<u32>().unwrap();
        let r = line[8..13].parse::<u32>().unwrap();

        left.push(l);
        right.push(r);
    }

    left.sort_unstable();
    right.sort_unstable();

    let answer = left
        .into_iter()
        .zip(right)
        .map(|(l, r)| {
            let min = l.min(r);
            let max = l.max(r);
            max - min
        })
        .sum::<u32>();

    Some(Box::new(answer))
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut left = HashMap::<u32, u32>::new();
    let mut right = HashMap::<u32, u32>::new();

    for line in input.lines() {
        let l = line[0..5].parse::<u32>().unwrap();
        let r = line[8..13].parse::<u32>().unwrap();

        let l_count = left.entry(l).or_default();
        *l_count += 1;

        let r_count = right.entry(r).or_default();
        *r_count += 1;
    }

    let answer = left
        .into_iter()
        .map(|(k, v)| right.get(&k).copied().unwrap_or_default() * v * k)
        .sum::<u32>();

    Some(Box::new(answer))
}

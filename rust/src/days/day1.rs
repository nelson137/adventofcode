pub(super) fn part1(input: &str) -> Box<dyn std::fmt::Display> {
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

    Box::new(answer)
}

pub(super) fn part2(input: &str) -> Box<dyn std::fmt::Display> {
    _ = input;

    Box::new("_")
}

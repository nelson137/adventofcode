use regex::Regex;

pub(super) fn part1(input: &str) -> Box<dyn std::fmt::Display> {
    let args_re = Regex::new(r"mul\((?<a>\d+),(?<b>\d+)\)").unwrap();

    let answer = args_re
        .captures_iter(input)
        .map(|cap| {
            let a = cap["a"].parse::<u32>().unwrap();
            let b = cap["b"].parse::<u32>().unwrap();
            a * b
        })
        .sum::<u32>();

    Box::new(answer)
}

pub(super) fn part2(input: &str) -> Box<dyn std::fmt::Display> {
    _ = input;

    Box::new("_")
}

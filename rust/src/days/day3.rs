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
    let args_re =
        Regex::new(r"(?<do>do\(\))|(?<dont>don't\(\))|(mul\((?<a>\d+),(?<b>\d+)\))").unwrap();

    let mut enabled = true;
    let mut sum = 0;

    for cap in args_re.captures_iter(input) {
        if cap.name("do").is_some() {
            enabled = true;
        } else if cap.name("dont").is_some() {
            enabled = false;
        } else if enabled {
            if let (Some(a), Some(b)) = (cap.name("a"), cap.name("b")) {
                let a = a.as_str().parse::<u32>().unwrap();
                let b = b.as_str().parse::<u32>().unwrap();
                sum += a * b;
            }
        }
    }

    Box::new(sum)
}

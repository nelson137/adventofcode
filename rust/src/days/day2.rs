pub(super) fn part1(input: &str) -> Box<dyn std::fmt::Display> {
    let answer = input.lines().filter(is_report_safe).count();
    Box::new(answer)
}

fn is_report_safe(line: &&str) -> bool {
    let levels = line.split(' ').map(|lvl| lvl.parse::<i32>().unwrap());

    #[derive(Debug, PartialEq)]
    enum LvlDelta {
        Inc,
        Dec,
        Unsafe,
    }

    let mut level_deltas = levels.map_windows(|[a, b]| match *b - *a {
        -3..=-1 => LvlDelta::Dec,
        1..=3 => LvlDelta::Inc,
        _ => LvlDelta::Unsafe,
    });

    let delta = level_deltas.next().unwrap();
    if delta == LvlDelta::Unsafe {
        return false;
    }

    level_deltas.all(|d| d == delta)
}

pub(super) fn part2(input: &str) -> Box<dyn std::fmt::Display> {
    _ = input;

    Box::new("_")
}

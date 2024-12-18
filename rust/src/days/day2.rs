crate::day_executors! {
    [part1]
    [part2]
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LvlDelta {
    Inc,
    Dec,
    Unsafe,
}

fn report_to_level_deltas(report: &str) -> impl Iterator<Item = LvlDelta> {
    report
        .split(' ')
        .map(|lvl| lvl.parse::<i32>().unwrap())
        .map_windows(|[a, b]| match *b - *a {
            -3..=-1 => LvlDelta::Dec,
            1..=3 => LvlDelta::Inc,
            _ => LvlDelta::Unsafe,
        })
}

pub(super) fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut safe_count = 0;

    for mut level_deltas in input.lines().map(report_to_level_deltas) {
        let delta = level_deltas.next().unwrap();
        if delta == LvlDelta::Unsafe {
            continue;
        }

        if level_deltas.all(|d| d == delta) {
            safe_count += 1;
        }
    }

    Some(Box::new(safe_count))
}

pub(super) fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut safe_count = 0;

    'reports: for level_deltas in input.lines().map(report_to_level_deltas) {
        let mut did_dampen = false;
        let mut delta = LvlDelta::Unsafe;

        for d in level_deltas {
            if d == LvlDelta::Unsafe || (delta != LvlDelta::Unsafe && d != delta) {
                if did_dampen {
                    continue 'reports;
                } else {
                    did_dampen = true;
                }
            }
            if delta == LvlDelta::Unsafe {
                delta = d;
            }
        }

        safe_count += 1;
    }

    Some(Box::new(safe_count))
}

use anyhow::Result;
use criterion::{BenchmarkId, Criterion};

pub(crate) type DayPartAnswer = Box<dyn ::std::fmt::Display>;
type DayPartExecutor = for<'input> fn(&'input str) -> Option<DayPartAnswer>;

macro_rules! day_modules {
    ($( $day:ident ),+ $(,)?) => {
        $(
            mod $day;
        )+

        pub(crate) static CLI_DAY_VALUES: &[&str] = &[$(
            stringify!($day)
        ),+];

        static DAY_EXECUTORS: &[(DayPartExecutor, DayPartExecutor)] = &[$(
            (self::$day::part1, self::$day::part2)
        ),+];
    };
}

day_modules![day1, day2, day3, day4, day5];

pub(crate) fn execute_day(
    day_i: u32,
    input: String,
) -> Result<(Option<DayPartAnswer>, Option<DayPartAnswer>)> {
    let executors = DAY_EXECUTORS[(day_i - 1) as usize];

    let answer1 = (executors.0)(&input);
    let answer2 = (executors.1)(&input);

    Ok((answer1, answer2))
}

pub(crate) fn bench_day(c: &mut Criterion, day_i: u32, input: String) {
    let executors = DAY_EXECUTORS[(day_i - 1) as usize];

    let mut group = c.benchmark_group(format!("Day-{day_i}"));

    group.bench_with_input(
        BenchmarkId::new("Part-1", "puzzle-input"),
        &*input,
        |b, i| b.iter(|| (executors.0)(i)),
    );

    group.bench_with_input(
        BenchmarkId::new("Part-2", "puzzle-input"),
        &*input,
        |b, i| b.iter(|| (executors.1)(i)),
    );

    group.finish();
}

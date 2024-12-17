use anyhow::Result;
use criterion::{BenchmarkId, Criterion};

pub(crate) type DayPartAnswer = Box<dyn ::std::fmt::Display>;
type DayPartExecutorFn = for<'input> fn(&'input str) -> Option<DayPartAnswer>;
type DayPartExecutors = &'static [(&'static str, DayPartExecutorFn)];
type DayExecutors = (DayPartExecutors, DayPartExecutors);

macro_rules! day_modules {
    ($( $day:ident ),+ $(,)?) => {
        $(
            mod $day;
        )+

        pub(crate) static CLI_DAY_VALUES: &[&str] = &[$(
            stringify!($day)
        ),+];

        static DAY_EXECUTORS: &[DayExecutors] = &[$(
            self::$day::EXECUTORS
        ),+];
    };
}

#[macro_export]
macro_rules! day_executors {
    (
        [$( $ex1:ident ),+ $(,)?]
        [$( $ex2:ident ),+ $(,)?]
    ) => {
        pub(super) static EXECUTORS: super::DayExecutors = (
            &[$( (stringify!($ex1), $ex1) ),+],
            &[$( (stringify!($ex2), $ex2) ),+],
        );
    };
}

day_modules![day1, day2, day3, day4, day5, day6];

pub(crate) fn execute_day(
    day_i: u32,
    input: String,
) -> Result<(Option<DayPartAnswer>, Option<DayPartAnswer>)> {
    let executors = DAY_EXECUTORS[(day_i - 1) as usize];

    let answer1 = (executors.0[0].1)(&input);
    let answer2 = (executors.1[0].1)(&input);

    Ok((answer1, answer2))
}

pub(crate) fn bench_day(c: &mut Criterion, day_i: u32, input: String) {
    let day_executors = DAY_EXECUTORS[(day_i - 1) as usize];

    {
        let mut group = c.benchmark_group(format!("Day-{day_i}/Part-1"));
        for executor in day_executors.0 {
            group.bench_with_input(
                BenchmarkId::new(executor.0, "puzzle-input"),
                &*input,
                |b, i| b.iter(|| (executor.1)(i)),
            );
        }
    }

    {
        let mut group = c.benchmark_group(format!("Day-{day_i}/Part-2"));
        for executor in day_executors.1 {
            group.bench_with_input(
                BenchmarkId::new(executor.0, "puzzle-input"),
                &*input,
                |b, i| b.iter(|| (executor.1)(i)),
            );
        }
    }
}

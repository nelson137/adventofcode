use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use criterion::{BenchmarkId, Criterion};

pub(crate) type DayPartAnswer = Box<dyn ::std::fmt::Display>;

pub(crate) struct DayPartResult {
    pub(crate) answer: DayPartAnswer,
    pub(crate) duration: Duration,
}

pub(crate) struct DayResult(
    pub(crate) Option<DayPartResult>,
    pub(crate) Option<DayPartResult>,
);

type DayPartExecutorFn = for<'input> fn(&'input str) -> Option<DayPartAnswer>;
type DayPartExecutors = &'static [(&'static str, DayPartExecutorFn)];
type DayExecutors = (DayPartExecutors, DayPartExecutors);

type DayPartVisualizerFn = DayPartExecutorFn;
type DayVisualizers = (Option<DayPartVisualizerFn>, Option<DayPartVisualizerFn>);

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

        static DAY_VISUALIZERS: &[DayVisualizers] = &[$(
            self::$day::VISUALIZERS
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

#[macro_export]
macro_rules! day_visualizers {
    ([           ] [           ]) => {
        pub(super) static VISUALIZERS: super::DayVisualizers = (None, None);
    };
    ([$viz1:ident] [           ]) => {
        pub(super) static VISUALIZERS: super::DayVisualizers = (Some($viz1), None);
    };
    ([           ] [$viz2:ident]) => {
        pub(super) static VISUALIZERS: super::DayVisualizers = (None, Some($viz2));
    };
    ([$viz1:ident] [$viz2:ident]) => {
        pub(super) static VISUALIZERS: super::DayVisualizers = (Some($viz1), Some($viz2));
    };
}

day_modules![
    day1, day2, day3, day4, day5, day6, day7, day8, day9, day10, day11, day12, day13, day14, day15,
    day16
];

pub(crate) fn execute_day(day_i: u32, part1: bool, part2: bool, input: String) -> DayResult {
    let executors = DAY_EXECUTORS[(day_i - 1) as usize];

    let run_part = |should_run: bool, part: DayPartExecutorFn| -> Option<DayPartResult> {
        if should_run {
            let t = Instant::now();
            let answer = part(&input);
            let duration = t.elapsed();
            answer.map(|answer| DayPartResult { answer, duration })
        } else {
            None
        }
    };

    let part1_result = run_part(part1, executors.0[0].1);
    let part2_result = run_part(part2, executors.1[0].1);
    DayResult(part1_result, part2_result)
}

pub(crate) fn bench_day(
    c: &mut Criterion,
    day_i: u32,
    part1: bool,
    part2: bool,
    input: String,
) -> Result<()> {
    let day_executors = DAY_EXECUTORS[(day_i - 1) as usize];

    let (commit1, commit2) = crate::commit::get_existing_commits(day_i)?;

    if part1 {
        let mut group = c.benchmark_group(format!("Day{day_i}-Pt1"));
        for &(name, run) in day_executors.0 {
            if let (Some(c), Some(answer)) = (&commit1, run(&input)) {
                if crate::commit::DayPartCommit::new(&answer) != *c {
                    bail!("incorrect bench impl :(");
                }
            }
            let id = BenchmarkId::new(name, "in");
            group.bench_with_input(id, &*input, |b, i| b.iter(|| run(i)));
        }
    }

    if part2 {
        let mut group = c.benchmark_group(format!("Day{day_i}-Pt2"));
        for &(name, run) in day_executors.1 {
            if let (Some(c), Some(answer)) = (&commit2, run(&input)) {
                if crate::commit::DayPartCommit::new(&answer) != *c {
                    bail!("incorrect bench impl :(");
                }
            }
            let id = BenchmarkId::new(name, "in");
            group.bench_with_input(id, &*input, |b, i| b.iter(|| run(i)));
        }
    }

    Ok(())
}

pub(crate) fn get_day_visualizers(day_i: u32) -> DayVisualizers {
    DAY_VISUALIZERS[(day_i - 1) as usize]
}

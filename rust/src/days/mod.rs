use std::time::{Duration, Instant};

use anyhow::Result;
use criterion::{BenchmarkId, Criterion};

pub(crate) type DayPartAnswer = Box<dyn ::std::fmt::Display>;

pub(crate) struct DayPartResult {
    pub(crate) answer: Option<DayPartAnswer>,
    pub(crate) duration: Duration,
}

pub(crate) struct DayResult(pub(crate) DayPartResult, pub(crate) DayPartResult);

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

day_modules![day1, day2, day3, day4, day5, day6, day7, day8];

pub(crate) fn execute_day(day_i: u32, input: String) -> Result<DayResult> {
    let executors = DAY_EXECUTORS[(day_i - 1) as usize];

    let i1 = Instant::now();
    let answer1 = (executors.0[0].1)(&input);
    let d1 = i1.elapsed();

    let i2 = Instant::now();
    let answer2 = (executors.1[0].1)(&input);
    let d2 = i2.elapsed();

    Ok(DayResult(
        DayPartResult {
            answer: answer1,
            duration: d1,
        },
        DayPartResult {
            answer: answer2,
            duration: d2,
        },
    ))
}

pub(crate) fn bench_day(c: &mut Criterion, day_i: u32, input: String) {
    let day_executors = DAY_EXECUTORS[(day_i - 1) as usize];

    {
        let mut group = c.benchmark_group(format!("Day{day_i}-Pt1"));
        for &(name, run) in day_executors.0 {
            let id = BenchmarkId::new(name, "in");
            group.bench_with_input(id, &*input, |b, i| b.iter(|| run(i)));
        }
    }

    {
        let mut group = c.benchmark_group(format!("Day{day_i}-Pt2"));
        for &(name, run) in day_executors.1 {
            let id = BenchmarkId::new(name, "in");
            group.bench_with_input(id, &*input, |b, i| b.iter(|| run(i)));
        }
    }
}

pub(crate) fn get_day_visualizers(day_i: u32) -> DayVisualizers {
    DAY_VISUALIZERS[(day_i - 1) as usize]
}

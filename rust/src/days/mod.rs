use std::{
    fmt,
    time::{Duration, Instant},
};

use adventofcode as aoc;
use anyhow::{Result, bail};
use criterion::{BenchmarkId, Criterion};
use crossterm::style;

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

day_modules![
    day1, day2, day3, day4, day5, day6, day7, day8, day9, day10, day11, day12, day13, day14, day15,
    day16
];

type DayExecutors = (&'static [DayPartExecutor], &'static [DayPartExecutor]);
type DayPartExecutorFn = for<'input> fn(&'input str) -> Option<DayPartAnswer>;

pub struct DayPartExecutor {
    name: &'static str,
    executor: DayPartExecutorFn,
}

impl DayPartExecutor {
    pub const fn new(name: &'static str, executor: DayPartExecutorFn) -> Self {
        Self { name, executor }
    }
}

pub(crate) type DayPartAnswer = Box<dyn ::std::fmt::Display>;

pub(crate) struct DayResult(
    pub(crate) Option<DayPartResult>,
    pub(crate) Option<DayPartResult>,
);

pub(crate) struct DayPartResult {
    pub(crate) answer: DayPartAnswer,
    pub(crate) duration: DayPartDuration,
}

pub(crate) struct DayPartDuration(Duration);

impl DayPartDuration {
    pub(crate) fn speed_color(&self) -> style::Color {
        if self.0 < Duration::from_millis(5) {
            style::Color::Grey
        } else if self.0 < Duration::from_millis(50) {
            style::Color::Yellow
        } else {
            style::Color::Red
        }
    }
}

impl fmt::Display for DayPartDuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[rustfmt::skip]
        #[allow(clippy::identity_op)]
        const MAX_WIDTH: usize =
            1 /* '('    */ +
            3 /* int    */ +
            1 /* '.'    */ +
            2 /* fract  */ +
            2 /* suffix */ +
            1 /* ')'    */ +
            0;

        const NANOS_PER_MILLI: u32 = 1_000_000;
        const NANOS_PER_MICRO: u32 = 1_000;

        let mut chars_written = 0_u64;

        if f.alternate() {
            write!(f, "{}(", style::SetForegroundColor(style::Color::DarkGrey))?;
            chars_written += 1;
        }

        fn fmt_decimal(
            f: &mut fmt::Formatter,
            int: u32,
            fract: u32,
            suffix: &str,
        ) -> ::std::result::Result<u64, fmt::Error> {
            let mut chars_written = 0;

            let fract_rounded = (fract + 5) / 10;
            if fract_rounded > 0 {
                write!(f, "{int}.{fract_rounded:02}{suffix}")?;
                chars_written += aoc::count_digits(int as u64);
                chars_written += 1 /* '.' */;
                chars_written += 2 /* fract digits */;
                chars_written += suffix.chars().count() as u64;
            } else {
                write!(f, "{int}{suffix}")?;
                chars_written += aoc::count_digits(int as u64);
                chars_written += suffix.chars().count() as u64;
            }

            Ok(chars_written)
        }

        if self.0.as_secs() > 0 {
            let int = self.0.as_secs();
            if int > 999 {
                write!(f, ">999s")?;
                chars_written += 5;
            } else {
                let fract = self.0.subsec_millis();
                chars_written += fmt_decimal(f, int as u32, fract, "s")?;
            }
        } else if self.0.subsec_nanos() >= NANOS_PER_MILLI {
            let int = self.0.subsec_millis();
            let fract = (self.0.subsec_nanos() % NANOS_PER_MILLI) / NANOS_PER_MICRO;
            chars_written += fmt_decimal(f, int, fract, "ms")?;
        } else if self.0.subsec_nanos() >= NANOS_PER_MICRO {
            let int = self.0.subsec_micros();
            let fract = self.0.subsec_nanos() % NANOS_PER_MICRO;
            chars_written += fmt_decimal(f, int, fract, "µs")?;
        } else {
            let int = self.0.subsec_nanos();
            let fract = 0;
            chars_written += fmt_decimal(f, int, fract, "ns")?;
        }

        if f.alternate() {
            write!(f, "){}", style::SetForegroundColor(style::Color::Reset))?;
            chars_written += 1;
        }

        let chars_remaining = if let Some(width) = f.width() {
            width
        } else if f.alternate() {
            MAX_WIDTH
        } else {
            0
        }
        .saturating_sub(chars_written as usize);

        if chars_remaining > 0 {
            write!(f, "{:width$}", "", width = chars_remaining)?;
        }

        Ok(())
    }
}

type DayVisualizers = (Option<DayPartVisualizerFn>, Option<DayPartVisualizerFn>);
type DayPartVisualizerFn = DayPartExecutorFn;

#[macro_export]
macro_rules! day_executors {
    (
        [$( $ex1:ident ),+ $(,)?]
        [$( $ex2:ident ),+ $(,)?]
    ) => {
        pub(super) static EXECUTORS: super::DayExecutors = (
            &[$( super::DayPartExecutor::new(stringify!($ex1), $ex1) ),+],
            &[$( super::DayPartExecutor::new(stringify!($ex2), $ex2) ),+],
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

pub(crate) fn execute_day(day_i: u32, part1: bool, part2: bool, input: String) -> DayResult {
    let executors = DAY_EXECUTORS[(day_i - 1) as usize];

    let run_part = |should_run: bool, part: DayPartExecutorFn| -> Option<DayPartResult> {
        if should_run {
            let t = Instant::now();
            let answer = part(&input);
            let duration = t.elapsed();
            answer.map(|answer| DayPartResult {
                answer,
                duration: DayPartDuration(duration),
            })
        } else {
            None
        }
    };

    let part1_result = run_part(part1, executors.0[0].executor);
    let part2_result = run_part(part2, executors.1[0].executor);
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

    let crate::commit::DayCommits(commit1, commit2) = crate::commit::get_existing_commits(day_i)?;

    if part1 {
        let mut group = c.benchmark_group(format!("Day{day_i}-Pt1"));
        for e in day_executors.0 {
            match (&commit1, (e.executor)(&input)) {
                (Some(_), None) => bail!("incorrect bench impl :("),
                (Some(c), Some(answer)) if crate::commit::DayPartCommit::new(&answer) != *c => {
                    bail!("incorrect bench impl :(")
                }
                _ => {}
            }
            let id = BenchmarkId::new(e.name, "in");
            group.bench_with_input(id, &*input, |b, i| b.iter(|| (e.executor)(i)));
        }
    }

    if part2 {
        let mut group = c.benchmark_group(format!("Day{day_i}-Pt2"));
        for e in day_executors.1 {
            match (&commit2, (e.executor)(&input)) {
                (Some(_), None) => bail!("incorrect bench impl :("),
                (Some(c), Some(answer)) if crate::commit::DayPartCommit::new(&answer) != *c => {
                    bail!("incorrect bench impl :(")
                }
                _ => {}
            }
            let id = BenchmarkId::new(e.name, "in");
            group.bench_with_input(id, &*input, |b, i| b.iter(|| (e.executor)(i)));
        }
    }

    Ok(())
}

pub(crate) fn get_day_visualizers(day_i: u32) -> DayVisualizers {
    DAY_VISUALIZERS[(day_i - 1) as usize]
}

use std::{
    collections::HashMap,
    fmt,
    time::{Duration, Instant},
};

use adventofcode as aoc;
use anyhow::{Result, bail};
use criterion::{BenchmarkId, Criterion};
use crossterm::style;

mod day1;
mod day10;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;
mod day16;
mod day2;
mod day3;
mod day4;
mod day5;
mod day6;
mod day7;
mod day8;
mod day9;

pub struct DayModule {
    id: u32,
    executors: DayExecutors,
    visualizers: DayVisualizers,
}

impl DayModule {
    pub const fn new(id: u32) -> Self {
        Self {
            id,
            executors: (&[], &[]),
            visualizers: (None, None),
        }
    }

    pub const fn with_executors(
        mut self,
        pt1_executors: &'static [DayPartExecutor],
        pt2_executors: &'static [DayPartExecutor],
    ) -> Self {
        self.executors = (pt1_executors, pt2_executors);
        self
    }

    pub const fn with_pt1_visualizer(mut self, visualizer: DayPartVisualizerFn) -> Self {
        self.visualizers.0 = Some(visualizer);
        self
    }

    pub const fn with_pt2_visualizer(mut self, visualizer: DayPartVisualizerFn) -> Self {
        self.visualizers.1 = Some(visualizer);
        self
    }
}

#[macro_export]
macro_rules! day_part_executors {
    ( $( $ex:ident ),+ $(,)? ) => {
        &[$( $crate::days::DayPartExecutor::new(stringify!($ex), $ex) ),+]
    };
}

inventory::collect!(DayModule);

pub(crate) static DAY_IDS: std::sync::LazyLock<Vec<u32>> =
    std::sync::LazyLock::new(|| inventory::iter::<DayModule>().map(|m| m.id).collect());

static DAY_EXECUTORS: std::sync::LazyLock<HashMap<u32, DayExecutors>> =
    std::sync::LazyLock::new(|| {
        inventory::iter::<DayModule>()
            .map(|m| (m.id, m.executors))
            .collect()
    });

static DAY_VISUALIZERS: std::sync::LazyLock<HashMap<u32, DayVisualizers>> =
    std::sync::LazyLock::new(|| {
        inventory::iter::<DayModule>()
            .map(|m| (m.id, m.visualizers))
            .collect()
    });

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

pub(crate) fn execute_day(day_i: u32, part1: bool, part2: bool, input: String) -> DayResult {
    let executors = DAY_EXECUTORS[&day_i];

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
    let day_executors = DAY_EXECUTORS[&day_i];

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
    DAY_VISUALIZERS[&day_i]
}

use std::{
    collections::HashMap,
    fmt,
    time::{Duration, Instant},
};

use adventofcode as aoc;
use anyhow::{Result, bail};
use criterion::{BenchmarkId, Criterion};
use crossterm::style;

type YearAndDay = (u32, u32);

pub struct DayModule {
    key: YearAndDay,
    executors: DayExecutors,
    visualizers: DayVisualizers,
}

impl DayModule {
    pub const fn new(year: u32, day: u32) -> Self {
        Self {
            key: (year, day),
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

static DAY_EXECUTORS: std::sync::LazyLock<HashMap<YearAndDay, DayExecutors>> =
    std::sync::LazyLock::new(|| {
        inventory::iter::<DayModule>()
            .map(|m| (m.key, m.executors))
            .collect()
    });

static DAY_VISUALIZERS: std::sync::LazyLock<HashMap<YearAndDay, DayVisualizers>> =
    std::sync::LazyLock::new(|| {
        inventory::iter::<DayModule>()
            .map(|m| (m.key, m.visualizers))
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

#[derive(Default)]
pub(crate) struct DayResult(
    pub(crate) Vec<Option<DayPartResult>>,
    pub(crate) Vec<Option<DayPartResult>>,
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

pub(crate) fn execute_day(
    year: u32,
    day_i: u32,
    part1: bool,
    part2: bool,
    input: String,
) -> Option<DayResult> {
    let executors = DAY_EXECUTORS.get(&(year, day_i))?;

    let run_part = |should_run: bool, day: &[DayPartExecutor]| -> Vec<Option<DayPartResult>> {
        if should_run {
            day.iter()
                .map(|d| {
                    let t = Instant::now();
                    let answer = (d.executor)(&input);
                    let duration = t.elapsed();
                    answer.map(|answer| DayPartResult {
                        answer,
                        duration: DayPartDuration(duration),
                    })
                })
                .collect()
        } else {
            vec![None]
        }
    };

    let part1_result = run_part(part1, executors.0);
    let part2_result = run_part(part2, executors.1);
    Some(DayResult(part1_result, part2_result))
}

pub(crate) fn bench_day(
    c: &mut Criterion,
    year: u32,
    day_i: u32,
    part1: bool,
    part2: bool,
    input: String,
) -> Option<Result<()>> {
    let day_executors = DAY_EXECUTORS.get(&(year, day_i))?;
    Some(bench_day_inner(
        c,
        day_executors,
        year,
        day_i,
        part1,
        part2,
        input,
    ))
}

fn bench_day_inner(
    c: &mut Criterion,
    day_executors: &DayExecutors,
    year: u32,
    day_i: u32,
    part1: bool,
    part2: bool,
    input: String,
) -> Result<()> {
    let crate::commit::DayCommits(commit1, commit2) =
        crate::commit::get_existing_commits(year, day_i)?;

    if part1 {
        let mut group = c.benchmark_group(format!("Year{year}-Day{day_i}-Pt1"));
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

pub(crate) fn get_day_visualizers(year: u32, day_i: u32) -> Option<&'static DayVisualizers> {
    DAY_VISUALIZERS.get(&(year, day_i))
}

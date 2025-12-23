#![feature(iter_map_windows)]

use std::{fmt, iter};

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand};
use crossterm::style::{self, StyledContent, Stylize};

mod auth;
mod commit;
mod days;
mod input;
mod puzzles;

pub(crate) static PUZZLE_DIR: &str = env!("PUZZLE_DIR");

fn main() -> Result<()> {
    Cli::parse().run()
}

// ###################################################################
// # CLI
// ###################################################################

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
enum Cli {
    Auth {
        #[command(subcommand)]
        command: CliAuthCommand,
    },
    Bench(CliBenchCommand),
    Commit(CliCommitCommand),
    Input {
        #[command(subcommand)]
        command: CliInputCommand,
    },
    Run(CliRunCommand),
    #[command(alias = "viz")]
    Visualize(CliVisualizeCommand),
}

impl Cli {
    fn run(self) -> Result<()> {
        match self {
            Self::Auth { command } => command.run(),
            Self::Bench(command) => command.run(),
            Self::Commit(command) => command.run(),
            Self::Input { command } => command.run(),
            Self::Run(command) => command.run(),
            Self::Visualize(command) => command.run(),
        }
    }
}

// ###################################################################
// # CLI - Auth
// ###################################################################

#[derive(Subcommand, Clone, Debug)]
enum CliAuthCommand {
    Get {
        #[arg(long)]
        location: bool,
    },
    Set,
}

impl CliAuthCommand {
    fn run(self) -> Result<()> {
        match self {
            Self::Get { location } => {
                if location {
                    println!("{}", auth::TOKEN_PATH.display());
                } else {
                    let token = auth::get_token()?;
                    println!("{}", token.trim());
                }
            }
            Self::Set => {
                if let Some(token) = auth::prompt_for_token()? {
                    auth::set_token(token)?;
                }
            }
        }
        Ok(())
    }
}

// ###################################################################
// # CLI - Bench
// ###################################################################

#[derive(Args, Clone, Debug)]
struct CliBenchCommand {
    #[command(flatten)]
    parts: CliDefaultedPartsGroup,

    #[arg(value_parser = YearParser::new())]
    year: Year,

    #[arg(value_parser = DayParser)]
    day: Day,
}

impl CliBenchCommand {
    fn run(self) -> Result<()> {
        let input = input::get_input(self.year.0, self.day.0)?;
        let mut criterion = criterion::Criterion::default();
        if days::bench_day(
            &mut criterion,
            self.year.0,
            self.day.0,
            self.parts.part1(),
            self.parts.part2(),
            input,
        )
        .is_none()
        {
            println!(
                "No implementation for year {} day {}.",
                &self.year.0, self.day.0
            );
        }

        criterion::Criterion::default().final_summary();

        Ok(())
    }
}

// ###################################################################
// # CLI - Commit
// ###################################################################

#[derive(Args, Clone, Debug)]
struct CliCommitCommand {
    #[arg(long)]
    force: bool,

    #[arg(value_parser = YearParser::new())]
    year: Year,

    #[arg(value_parser = DayParser)]
    day: Day,
}

impl CliCommitCommand {
    fn run(self) -> Result<()> {
        let input = input::get_input(self.year.0, self.day.0)?;

        let Some(result) = days::execute_day(self.year.0, self.day.0, true, true, input) else {
            println!(
                "No implementation for year {} day {}.",
                &self.year.0, self.day.0
            );
            return Ok(());
        };

        let existing_commits = commit::get_existing_commits(self.year.0, self.day.0)?;

        if let Some(result1) = result.0 {
            let commit1 = commit::DayPartCommit::new(&result1.answer);
            match existing_commits.0 {
                Some(existing1) if commit1 == existing1 => {
                    self.print_already_committed(Part::Part1, &commit1.answer);
                }
                Some(existing1) if !self.force => {
                    self.print_incorrect_answer_diff(
                        Part::Part1,
                        &existing1.answer,
                        &commit1.answer,
                    );
                }
                _ => {
                    commit1.write(self.year.0, self.day.0, Part::Part1)?;
                    self.print_committed(Part::Part1, &commit1.answer);
                }
            }
        }

        if let Some(result2) = result.1 {
            let commit2 = commit::DayPartCommit::new(&result2.answer);
            match existing_commits.1 {
                Some(existing2) if commit2 == existing2 => {
                    self.print_already_committed(Part::Part2, &commit2.answer);
                }
                Some(existing2) if !self.force => {
                    self.print_incorrect_answer_diff(
                        Part::Part2,
                        &existing2.answer,
                        &commit2.answer,
                    );
                }
                _ => {
                    commit2.write(self.year.0, self.day.0, Part::Part2)?;
                    self.print_committed(Part::Part2, &commit2.answer);
                }
            }
        }

        Ok(())
    }

    fn print_committed(&self, part: Part, answer: &str) {
        println!(
            "{}: {}  {}    {}",
            part,
            answer.trim(),
            "✓".bold().green(),
            "(committed)".dark_grey()
        );
    }

    fn print_already_committed(&self, part: Part, answer: &str) {
        println!(
            "{}: {}  {}    {}",
            part,
            answer.trim(),
            "✓".bold().green(),
            "(already committed)".dark_grey()
        );
    }

    fn print_incorrect_answer_diff(&self, part: Part, commit_answer: &str, current_answer: &str) {
        eprintln!(
            "{}: {} answer does not match existing commit",
            part,
            "error".bold().red(),
        );
        eprintln!();
        eprintln!("<<<<<<< commited answer");
        eprintln!("{}", commit_answer.trim());
        eprintln!("=======");
        eprintln!("{}", current_answer.trim());
        eprintln!(">>>>>>> current run answer");
        eprintln!();
        eprintln!("Use `--force` to overwrite");
        eprintln!();
    }
}

// ###################################################################
// # CLI - Input
// ###################################################################

#[derive(Subcommand, Clone, Debug)]
enum CliInputCommand {
    Get {
        #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "1")]
        test: Option<u32>,

        #[arg(value_parser = YearParser::new())]
        year: Year,

        #[arg(value_parser = DayParser)]
        day: Day,
    },
    SetTest {
        #[arg(value_parser = YearParser::new())]
        year: Year,

        #[arg(value_parser = DayParser)]
        day: Day,

        #[arg(default_value_t = 1)]
        test: u32,
    },
}

impl CliInputCommand {
    fn run(self) -> Result<()> {
        match self {
            Self::Get { test, year, day } => {
                if let Some(test_i) = test {
                    let day_test_input = input::get_test_input(year.0, day.0, test_i)?;
                    print!("{day_test_input}");
                } else {
                    let day_input = input::get_input(year.0, day.0)?;
                    print!("{day_input}");
                }
            }
            Self::SetTest { year, day, test } => {
                input::set_test_input(year.0, day.0, test)?;
            }
        }
        Ok(())
    }
}

// ###################################################################
// # CLI - Run
// ###################################################################

#[derive(Args, Clone, Debug)]
struct CliRunCommand {
    #[command(flatten)]
    parts: CliDefaultedPartsGroup,

    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "1")]
    test: Option<u32>,

    #[arg(name = "year", value_parser = YearParser::new())]
    year: Year,

    #[arg(name = "day", value_parser = CliRunDaySpecParser::new())]
    day_spec: CliRunDaySpec,
}

impl CliRunCommand {
    fn run(self) -> Result<()> {
        match self.day_spec {
            CliRunDaySpec::All => self.run_all(),
            CliRunDaySpec::Day(day) => self.run_one(day),
        }
    }

    fn run_one(self, day: Day) -> Result<()> {
        let input = if let Some(test_i) = self.test {
            input::get_test_input(self.year.0, day.0, test_i)?
        } else {
            input::get_input(self.year.0, day.0)?
        };

        let Some(result) = days::execute_day(
            self.year.0,
            day.0,
            self.parts.part1(),
            self.parts.part2(),
            input,
        ) else {
            println!("No implementation for year {} day {}.", &self.year.0, day.0);
            return Ok(());
        };

        let existing_commits = commit::get_existing_commits(self.year.0, day.0)?;

        if let Some(r1) = result.0 {
            let commit1 = commit::DayPartCommit::new(&r1.answer);
            let commit_status =
                self.get_single_day_commit_status(existing_commits.0.as_ref(), &commit1);
            println!(
                "Part 1: {answer}{status}    {duration:#}",
                answer = r1.answer,
                status = commit_status,
                duration = r1.duration,
            );
        }

        if let Some(r2) = result.1 {
            let commit2 = commit::DayPartCommit::new(&r2.answer);
            let commit_status =
                self.get_single_day_commit_status(existing_commits.1.as_ref(), &commit2);
            println!(
                "Part 2: {answer}{status}    {duration:#}",
                answer = r2.answer,
                status = commit_status,
                duration = r2.duration,
            );
        }

        Ok(())
    }

    fn get_single_day_commit_status(
        &self,
        existing_commit: Option<&commit::DayPartCommit>,
        current_commit: &commit::DayPartCommit,
    ) -> StyledContent<&'static str> {
        if self.test.is_some() {
            return "".stylize();
        }
        match existing_commit {
            Some(existing) if current_commit == existing => "  ✔".bold().green(),
            Some(_) => "  ✗".bold().red(),
            None => "".stylize(),
        }
    }

    fn run_all(self) -> Result<()> {
        if self.test.is_some() {
            eprintln!("Warning: ignoring `--test` option, it does nothing with `run all`");
        }

        const GUTTER_WIDTH: usize = 4;
        const PADDING_WIDTH: usize = 4;
        const STATUS_WIDTH: usize = 3;
        const DURATION_WIDTH: usize = 8;
        const PART_WIDTH: usize = STATUS_WIDTH + DURATION_WIDTH;

        let results = (1_u32..=25)
            .map(
                |day_i| -> Result<(u32, commit::DayCommits, days::DayResult)> {
                    let commits = commit::get_existing_commits(self.year.0, day_i)?;
                    let input = input::get_input(self.year.0, day_i)?;
                    let result = days::execute_day(self.year.0, day_i, true, true, input)
                        .unwrap_or_default();
                    Ok((day_i, commits, result))
                },
            )
            .collect::<Result<Vec<_>>>()?;

        println!(
            "{spacer:gutter_w$} {spacer:padding_w$}{p1}{spacer:padding_w$}{p2}",
            spacer = "",
            gutter_w = GUTTER_WIDTH,
            padding_w = PADDING_WIDTH,
            p1 = format!("{:w$}", "Pt. 1", w = PART_WIDTH).grey(),
            p2 = format!("{:w$}", "Pt. 2", w = PART_WIDTH).grey(),
        );

        println!(
            "{gutter}{divider}{padding}{part}{padding}{part}{tail}",
            gutter = "─".repeat(GUTTER_WIDTH).dark_grey(),
            divider = "┬".dark_grey(),
            padding = "─".repeat(PADDING_WIDTH).dark_grey(),
            part = "─".repeat(PART_WIDTH).dark_grey(),
            tail = "─".repeat(PADDING_WIDTH).dark_grey(),
        );

        for (day_i, day_commits, day_result) in results {
            print!(
                " {day_label} {divider}",
                day_label = format!("{day_i:2}").grey(),
                divider = "│".dark_grey(),
            );

            for (existing_commit, result) in [
                (day_commits.0.as_ref(), day_result.0.as_ref()),
                (day_commits.1.as_ref(), day_result.1.as_ref()),
            ] {
                if let Some(r) = result {
                    let commit = commit::DayPartCommit::new(&r.answer);
                    let commit_status =
                        self.get_many_day_commit_status(existing_commit, Some(&commit));
                    print!(
                        "{padding:padding_w$}{status}  {duration_fg}{duration:duration_w$}{reset}",
                        padding = "",
                        padding_w = PADDING_WIDTH,
                        status = commit_status,
                        duration = r.duration,
                        duration_fg = style::SetForegroundColor(r.duration.speed_color()),
                        duration_w = DURATION_WIDTH,
                        reset = style::SetForegroundColor(style::Color::Reset),
                    );
                } else {
                    let commit_status = self.get_many_day_commit_status(existing_commit, None);
                    print!(
                        "{spacer:padding_w$}{status}  {spacer:duration_width$}",
                        padding_w = PADDING_WIDTH,
                        status = commit_status,
                        spacer = "",
                        duration_width = DURATION_WIDTH
                    );
                }
            }

            println!();
        }

        Ok(())
    }

    fn get_many_day_commit_status(
        &self,
        existing_commit: Option<&commit::DayPartCommit>,
        current_commit: Option<&commit::DayPartCommit>,
    ) -> StyledContent<&'static str> {
        match (current_commit, existing_commit) {
            (None, _) => "★".dark_grey(),
            (Some(current), Some(existing)) if current == existing => "★".bold().yellow(),
            (Some(_), Some(_)) => "✗".bold().red(),
            (Some(_), None) => "?".bold().dark_magenta(),
        }
    }
}

// ###################################################################
// # CLI - Visualize
// ###################################################################

#[derive(Args, Clone, Debug)]
struct CliVisualizeCommand {
    #[command(flatten)]
    parts: CliVisualizeCommandPartsGroup,

    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "1")]
    test: Option<u32>,

    #[arg(value_parser = YearParser::new())]
    year: Year,

    #[arg(value_parser = DayParser)]
    day: Day,
}

#[derive(Args, Clone, Debug)]
#[group(required = true, multiple = false)]
struct CliVisualizeCommandPartsGroup {
    #[arg(long)]
    part1: bool,

    #[arg(long)]
    part2: bool,
}

impl CliVisualizeCommand {
    fn run(self) -> Result<()> {
        let Some(visualizers) = days::get_day_visualizers(self.year.0, self.day.0) else {
            println!(
                "No implementation for year {} day {}.",
                &self.year.0, self.day.0
            );
            return Ok(());
        };

        let (id, visualize) = if self.parts.part1 {
            (1, visualizers.0)
        } else {
            (2, visualizers.1)
        };

        let Some(visualize) = visualize else {
            bail!("there is no visualizer for Day {} Part {id}", self.day.0);
        };

        let input = if let Some(test_i) = self.test {
            input::get_test_input(self.year.0, self.day.0, test_i)?
        } else {
            input::get_input(self.year.0, self.day.0)?
        };

        if let Some(answer) = visualize(&input) {
            println!("Part {id}: {answer}");
        }

        Ok(())
    }
}

// ###################################################################
// # Data
// ###################################################################

#[derive(Args, Clone, Debug)]
#[group(required = false, multiple = true)]
struct CliDefaultedPartsGroup {
    #[arg(long)]
    part1: bool,

    #[arg(long)]
    part2: bool,
}

impl CliDefaultedPartsGroup {
    fn none(&self) -> bool {
        !self.part1 && !self.part2
    }

    fn part1(&self) -> bool {
        self.part1 || self.none()
    }

    fn part2(&self) -> bool {
        self.part2 || self.none()
    }
}

#[derive(Clone, Debug)]
struct Year(u32);

const CLI_YEARS: &[&str] = &["2024", "2025"];

#[derive(Clone)]
struct YearParser {
    possible_values_parser: clap::builder::PossibleValuesParser,
}

impl YearParser {
    fn new() -> Self {
        Self {
            possible_values_parser: clap::builder::PossibleValuesParser::new(CLI_YEARS.to_vec()),
        }
    }
}

impl clap::builder::TypedValueParser for YearParser {
    type Value = Year;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> std::result::Result<Self::Value, clap::Error> {
        let year_raw = self.possible_values_parser.parse_ref(cmd, arg, value)?;
        match year_raw.parse() {
            Ok(y) => Ok(Year(y)),
            Err(_) => Err(clap::Error::new(clap::error::ErrorKind::InvalidValue)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Day(u32);

fn day_id_to_cli(n: u32) -> &'static str {
    format!("day{n}").leak() as &'static str
}

#[derive(Clone)]
struct DayParser;

impl clap::builder::TypedValueParser for DayParser {
    type Value = Day;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let possible_day_values =
            clap::builder::PossibleValuesParser::new((1..=25).map(day_id_to_cli));
        let day = possible_day_values.parse_ref(cmd, arg, value)?;
        let n = day[3..].parse().unwrap();
        Ok(Day(n))
    }
}

#[derive(Clone, Copy, Debug)]
enum CliRunDaySpec {
    All,
    Day(Day),
}

#[derive(Clone)]
struct CliRunDaySpecParser {
    possible_values_parser: clap::builder::PossibleValuesParser,
}

impl CliRunDaySpecParser {
    fn new() -> Self {
        let possible_values = iter::once("all")
            .chain((1..=25).map(day_id_to_cli))
            .collect::<Vec<_>>();
        let possible_values_parser = clap::builder::PossibleValuesParser::new(possible_values);
        Self {
            possible_values_parser,
        }
    }
}

impl clap::builder::TypedValueParser for CliRunDaySpecParser {
    type Value = CliRunDaySpec;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> std::result::Result<Self::Value, clap::Error> {
        let day_spec = self.possible_values_parser.parse_ref(cmd, arg, value)?;
        Ok(if day_spec == "all" {
            CliRunDaySpec::All
        } else {
            CliRunDaySpec::Day(Day(day_spec[3..].parse().unwrap()))
        })
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum Part {
    Part1 = 1,
    Part2 = 2,
}

impl Part {
    fn number(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Part {}", self.number())
    }
}

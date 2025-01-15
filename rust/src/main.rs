#![feature(iter_map_windows)]

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand};
use crossterm::style::Stylize;

mod auth;
mod commit;
mod days;
mod input;

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
                input::create_inputs_dir()?;
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

    #[arg(value_parser = DayParser)]
    day: Day,
}

impl CliBenchCommand {
    fn run(self) -> Result<()> {
        let input = input::get_input(self.day.0)?;
        let mut criterion = criterion::Criterion::default();
        days::bench_day(
            &mut criterion,
            self.day.0,
            self.parts.part1(),
            self.parts.part2(),
            input,
        )?;

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

    #[arg(value_parser = DayParser)]
    day: Day,
}

impl CliCommitCommand {
    fn run(self) -> Result<()> {
        let input = input::get_input(self.day.0)?;

        let result = days::execute_day(self.day.0, true, true, input);

        let existing_commits = commit::get_existing_commits(self.day.0)?;

        if let Some(result1) = result.0 {
            let commit1 = commit::DayPartCommit::new(&result1.answer);
            match existing_commits.0 {
                Some(existing1) if commit1 == existing1 => {
                    self.print_already_committed(1, &commit1.answer);
                }
                Some(existing1) if !self.force => {
                    self.print_incorrect_answer_diff(1, &existing1.answer, &commit1.answer);
                }
                _ => {
                    commit1.write(self.day.0, 1)?;
                    self.print_committed(1, &commit1.answer);
                }
            }
        }

        if let Some(result2) = result.1 {
            let commit2 = commit::DayPartCommit::new(&result2.answer);
            match existing_commits.1 {
                Some(existing2) if commit2 == existing2 => {
                    self.print_already_committed(2, &commit2.answer);
                }
                Some(existing2) if !self.force => {
                    self.print_incorrect_answer_diff(2, &existing2.answer, &commit2.answer);
                }
                _ => {
                    commit2.write(self.day.0, 2)?;
                    self.print_committed(2, &commit2.answer);
                }
            }
        }

        Ok(())
    }

    fn print_committed(&self, part_i: u32, answer: &str) {
        println!(
            "Part {}: {}  {}    {}",
            part_i,
            answer.trim(),
            "✓".bold().green(),
            "(committed)".dark_grey()
        );
    }

    fn print_already_committed(&self, part_i: u32, answer: &str) {
        println!(
            "Part {}: {}  {}    {}",
            part_i,
            answer.trim(),
            "✓".bold().green(),
            "(already committed)".dark_grey()
        );
    }

    fn print_incorrect_answer_diff(&self, part_i: u32, commit_answer: &str, current_answer: &str) {
        eprintln!(
            "{}: part {part_i} answer does not match existing commit",
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

        #[arg(value_parser = DayParser)]
        day: Day,
    },
    SetTest {
        #[arg(value_parser = DayParser)]
        day: Day,

        #[arg(default_value_t = 1)]
        test: u32,
    },
}

impl CliInputCommand {
    fn run(self) -> Result<()> {
        match self {
            Self::Get { test, day } => {
                if let Some(test_i) = test {
                    let day_test_input = input::get_test_input(day.0, test_i)?;
                    print!("{day_test_input}");
                } else {
                    let day_input = input::get_input(day.0)?;
                    print!("{day_input}");
                }
            }
            Self::SetTest { day, test } => {
                input::set_test_input(day.0, test)?;
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

    #[arg(value_parser = DayParser)]
    day: Day,
}

impl CliRunCommand {
    fn run(self) -> Result<()> {
        let input = if let Some(test_i) = self.test {
            input::get_test_input(self.day.0, test_i)?
        } else {
            input::get_input(self.day.0)?
        };
        let result = days::execute_day(self.day.0, self.parts.part1(), self.parts.part2(), input);

        if let Some(r1) = result.0 {
            println!(
                "Part 1: {}  {}",
                r1.answer,
                format!("({:?})", r1.duration).dark_grey()
            );
        }

        if let Some(r2) = result.1 {
            println!(
                "Part 2: {}  {}",
                r2.answer,
                format!("({:?})", r2.duration).dark_grey()
            );
        }

        Ok(())
    }
}

// ###################################################################
// # CLI - Visualize
// ###################################################################

#[derive(Args, Clone, Debug)]
struct CliVisualizeCommand {
    #[command(flatten)]
    parts: CliVisualizeCommandPartsGroup,

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
        let visualizers = days::get_day_visualizers(self.day.0);

        let (id, visualize) = if self.parts.part1 {
            (1, visualizers.0)
        } else {
            (2, visualizers.1)
        };

        let Some(visualize) = visualize else {
            bail!("there is no visualizer for Day {} Part {id}", self.day.0);
        };

        let input = input::get_input(self.day.0)?;

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

#[derive(Clone, Copy, Debug)]
struct Day(u32);

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
        let possible_day_values = clap::builder::PossibleValuesParser::new(days::CLI_DAY_VALUES);
        let day = possible_day_values.parse_ref(cmd, arg, value)?;
        let n = day[3..].parse().unwrap();
        Ok(Day(n))
    }
}

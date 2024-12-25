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
}

impl Cli {
    fn run(self) -> Result<()> {
        match self {
            Self::Auth { command } => command.run(),
            Self::Bench(command) => command.run(),
            Self::Commit(command) => command.run(),
            Self::Input { command } => command.run(),
            Self::Run(command) => command.run(),
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
    #[arg(value_parser = DayParser)]
    day: Day,
}

impl CliBenchCommand {
    fn run(self) -> Result<()> {
        let input = input::get_input(self.day.0)?;
        let mut criterion = criterion::Criterion::default();
        days::bench_day(&mut criterion, self.day.0, input);

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

        let result = days::execute_day(self.day.0, input)?;
        let (Some(answer1), Some(answer2)) = (result.0.answer, result.1.answer) else {
            bail!("days can only be committed when both parts are solved");
        };

        let existing_commit = commit::get_existing_commit(self.day.0)?;
        let commit = commit::DayAnswersCommit::new(&answer1, &answer2);

        if let Some(existing_commit) = existing_commit {
            if commit == existing_commit {
                println!("Day {} is already commited", self.day.0);
                println!("Answers match");
            } else if self.force {
                println!(
                    "Part 1: {}  {}",
                    commit.answer1.trim(),
                    format!("({:?})", result.0.duration).dark_grey(),
                );
                println!(
                    "Part 2: {}  {}",
                    commit.answer2.trim(),
                    format!("({:?})", result.1.duration).dark_grey(),
                );
                commit::write_day_answers(self.day.0, &commit)?;
            } else {
                eprintln!(
                    "Day {} is already committed but the answers do not match",
                    self.day.0
                );
                eprintln!("Use `--force` to overwrite");
                if commit.answer1_checksum != existing_commit.answer1_checksum {
                    eprintln!();
                    eprintln!("Part 1:");
                    eprintln!("<<<<<<< commited answer");
                    eprintln!("{}", existing_commit.answer1.trim());
                    eprintln!("=======");
                    eprintln!("{}", commit.answer1.trim());
                    eprintln!(">>>>>>> current run answer");
                }
                if commit.answer2_checksum != existing_commit.answer2_checksum {
                    eprintln!();
                    eprintln!("Part 2:");
                    eprintln!("<<<<<<< commited answer");
                    eprintln!("{}", existing_commit.answer2.trim());
                    eprintln!("=======");
                    eprintln!("{}", commit.answer2.trim());
                    eprintln!(">>>>>>> current run answer");
                }
            }
        } else {
            println!("Part 1: {}", commit.answer1.trim());
            println!("Part 2: {}", commit.answer2.trim());
            commit::write_day_answers(self.day.0, &commit)?;
        }

        Ok(())
    }
}

// ###################################################################
// # CLI - Input
// ###################################################################

#[derive(Subcommand, Clone, Debug)]
enum CliInputCommand {
    Get {
        #[arg(long)]
        test: bool,

        #[arg(value_parser = DayParser)]
        day: Day,
    },
    SetTest {
        #[arg(value_parser = DayParser)]
        day: Day,
    },
}

impl CliInputCommand {
    fn run(self) -> Result<()> {
        match self {
            Self::Get { test, day } => {
                if test {
                    let day_test_input = input::get_test_input(day.0)?;
                    print!("{day_test_input}");
                } else {
                    let day_input = input::get_input(day.0)?;
                    print!("{day_input}");
                }
            }
            Self::SetTest { day } => {
                input::set_test_input(day.0)?;
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
    #[arg(long)]
    test: bool,

    #[arg(value_parser = DayParser)]
    day: Day,
}

impl CliRunCommand {
    fn run(self) -> Result<()> {
        let input = if self.test {
            input::get_test_input(self.day.0)?
        } else {
            input::get_input(self.day.0)?
        };
        let result = days::execute_day(self.day.0, input)?;

        if let Some(answer) = result.0.answer.as_deref() {
            println!(
                "Part 1: {answer}  {}",
                format!("({:?})", result.0.duration).dark_grey()
            );
        }

        if let Some(answer) = result.1.answer.as_deref() {
            println!(
                "Part 2: {answer}  {}",
                format!("({:?})", result.1.duration).dark_grey()
            );
        }

        Ok(())
    }
}

// ###################################################################
// # Data
// ###################################################################

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

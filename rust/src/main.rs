use anyhow::Result;
use clap::{Parser, Subcommand};

mod auth;
mod days;
mod inputs;

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
enum Cli {
    Auth {
        #[command(subcommand)]
        command: CliAuthCommand,
    },
    Run {
        #[arg(value_parser = DayParser)]
        day: Day,
    },
}

impl Cli {
    fn run(self) -> Result<()> {
        match self {
            Self::Auth { command } => command.run(),
            Self::Run { day } => days::execute_day(day.0),
        }
    }
}

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
                inputs::create_inputs_dir()?;
                if let Some(token) = auth::prompt_for_token()? {
                    auth::set_token(token)?;
                }
            }
        }
        Ok(())
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

fn main() -> Result<()> {
    Cli::parse().run()
}

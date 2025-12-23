use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::{Context, Result};
use reqwest::{
    blocking::{Client, Response},
    header as H,
};

pub(crate) static PUZZLE_INPUTS_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| Path::new(crate::PUZZLE_DIR).join("inputs"));

fn inputs_dir(year: u32) -> PathBuf {
    PUZZLE_INPUTS_DIR.join(year.to_string())
}

fn create_inputs_dir(year: u32) -> Result<PathBuf> {
    let dir = inputs_dir(year);
    fs::create_dir_all(&dir).with_context(|| {
        format!(
            "failed to create puzzle inputs directory: {}",
            dir.display()
        )
    })?;
    Ok(dir)
}

pub(crate) fn get_input(year: u32, day_i: u32) -> Result<String> {
    let dir = create_inputs_dir(year)?;
    let input_path = dir.join(format!("day{day_i}"));

    if input_path.exists() {
        fs::read_to_string(&input_path)
            .with_context(|| format!("failed to read puzzle input file: {}", input_path.display()))
    } else {
        let token = crate::auth::get_token()?;
        let token = token.trim();

        let url = format!("https://adventofcode.com/{year}/day/{day_i}/input");

        let input = Client::new()
            .get(&url)
            .header(H::COOKIE, format!("session={token}"))
            .send()
            .and_then(Response::error_for_status)
            .with_context(|| format!("failed to fetch puzzle input for day {day_i}"))?
            .text()
            .with_context(|| format!("failed to decode puzzle input: {url}"))?;

        fs::write(&input_path, &input).with_context(|| {
            format!(
                "failed to write puzzle input to file: {}",
                input_path.display()
            )
        })?;

        Ok(input)
    }
}

fn test_input_path(dir: &Path, day_i: u32, input_i: u32) -> PathBuf {
    dir.join(format!("day{day_i}-test{input_i}"))
}

pub(crate) fn get_test_input(year: u32, day_i: u32, input_i: u32) -> Result<String> {
    let dir = inputs_dir(year);
    let test_input_path = test_input_path(&dir, day_i, input_i);

    fs::read_to_string(&test_input_path).with_context(|| {
        format!(
            "failed to read puzzle test input file: {}",
            test_input_path.display()
        )
    })
}

pub(crate) fn set_test_input(year: u32, day_i: u32, input_i: u32) -> Result<()> {
    let dir = create_inputs_dir(year)?;

    println!("Enter test input below, press ^D when done");

    let mut test_input = Vec::<u8>::new();
    io::stdin()
        .read_to_end(&mut test_input)
        .with_context(|| "failed to read test input")?;

    let test_input_path = test_input_path(&dir, day_i, input_i);
    fs::write(&test_input_path, test_input).with_context(|| {
        format!(
            "failed to write puzzle test input to file: {}",
            test_input_path.display()
        )
    })?;

    Ok(())
}

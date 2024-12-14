use std::{fs, path::Path};

use anyhow::{Context, Result};
use reqwest::{
    blocking::{Client, Response},
    header as H,
};

pub(crate) static PUZZLE_INPUTS_DIR: &str = env!("PUZZLE_INPUTS_DIR");

pub(crate) fn create_inputs_dir() -> Result<()> {
    fs::create_dir_all(PUZZLE_INPUTS_DIR).with_context(|| {
        format!(
            "failed to create puzzle inputs directory: {}",
            PUZZLE_INPUTS_DIR
        )
    })
}

pub(crate) fn get_input(day_i: u32) -> Result<String> {
    let input_path = Path::new(PUZZLE_INPUTS_DIR).join(format!("day{day_i}"));

    if input_path.exists() {
        fs::read_to_string(&input_path)
            .with_context(|| format!("failed to read puzzle input file: {}", input_path.display()))
    } else {
        let token = crate::auth::get_token()?;
        let token = token.trim();

        let url = format!("https://adventofcode.com/2024/day/{day_i}/input");

        let input = Client::new()
            .get(&url)
            .header(H::COOKIE, format!("session={token}"))
            .send()
            .and_then(Response::text)
            .with_context(|| format!("failed to fetch puzzle input: {url}"))?;

        fs::write(&input_path, &input).with_context(|| {
            format!(
                "failed to write puzzle input to file: {}",
                input_path.display()
            )
        })?;

        Ok(input)
    }
}

use std::{fs, io::Write, path::PathBuf, sync::LazyLock};

use anyhow::{Context, Result};

mod term;

pub(crate) static TOKEN_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(crate::input::PUZZLE_INPUTS_DIR).join(".token"));

pub(crate) fn get_token() -> Result<String> {
    fs::read_to_string(&*TOKEN_PATH)
        .with_context(|| format!("failed to read GitHub PAT file: {}", TOKEN_PATH.display()))
}

pub(crate) fn prompt_for_token() -> Result<Option<String>> {
    term::read_token().with_context(|| "failed to read GitHub PAT")
}

pub(crate) fn set_token(token: String) -> Result<()> {
    let mut file = fs::File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&*TOKEN_PATH)?;
    writeln!(file, "{}", token.trim()).with_context(|| {
        format!(
            "failed to write GitHub PAT to file: {}",
            TOKEN_PATH.display()
        )
    })
}

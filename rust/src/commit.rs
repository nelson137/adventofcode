use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::{Context, Result, bail};

pub(crate) static PUZZLE_ANSWERS_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| Path::new(crate::PUZZLE_DIR).join("answers"));

pub(crate) struct DayCommits(
    pub(crate) Option<DayPartCommit>,
    pub(crate) Option<DayPartCommit>,
);

pub(crate) fn create_answers_dir() -> Result<()> {
    fs::create_dir_all(&*PUZZLE_ANSWERS_DIR).with_context(|| {
        format!(
            "failed to create puzzle answers directory: {}",
            PUZZLE_ANSWERS_DIR.display()
        )
    })
}

const CRC: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_BZIP2);

pub(crate) struct DayPartCommit {
    pub(crate) answer: String,
    pub(crate) checksum: u32,
}

impl PartialEq for DayPartCommit {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.checksum, &other.checksum)
    }
}

impl fmt::Display for DayPartCommit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let checksum = self.checksum;
        let answer = self.answer.trim();
        writeln!(f, "{checksum}")?;
        writeln!(f, "---")?;
        writeln!(f, "{answer}")?;
        Ok(())
    }
}

impl DayPartCommit {
    pub(crate) fn new(answer: &dyn fmt::Display) -> Self {
        let answer = answer.to_string();
        let checksum = CRC.checksum(answer.trim().as_bytes());
        Self { answer, checksum }
    }

    fn parse_from_file(path: impl AsRef<Path>) -> Result<Option<Self>> {
        match fs::read_to_string(&path) {
            Ok(commit) => Ok(Some(Self::parse(&commit).with_context(|| {
                format!("failed to parse commit file: {}", path.as_ref().display())
            })?)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err).with_context(|| {
                format!("failed to read commit file: {}", path.as_ref().display())
            })?,
        }
    }

    fn parse(commit_str: &str) -> Result<Self> {
        let Some((checksum, answer)) = commit_str.split_once("\n---\n") else {
            bail!("invalid commit format");
        };

        Ok(Self {
            answer: answer.to_string(),
            checksum: checksum.parse::<u32>()?,
        })
    }

    pub(crate) fn write(&self, day_i: u32, part: crate::Part) -> Result<()> {
        create_answers_dir()?;

        let path = PUZZLE_ANSWERS_DIR.join(format!("day{day_i}.{}", part.number()));
        let commit = self.to_string();

        fs::write(&path, commit)
            .with_context(|| format!("failed to write puzzle commit to file: {}", path.display()))
    }
}

pub(crate) fn get_existing_commits(day_i: u32) -> Result<DayCommits> {
    let commit1_path = PUZZLE_ANSWERS_DIR.join(format!("day{day_i}.1"));
    let commit2_path = PUZZLE_ANSWERS_DIR.join(format!("day{day_i}.2"));
    Ok(DayCommits(
        DayPartCommit::parse_from_file(&commit1_path)?,
        DayPartCommit::parse_from_file(&commit2_path)?,
    ))
}

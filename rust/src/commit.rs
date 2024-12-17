use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::{Context, Result, bail};

pub(crate) static PUZZLE_ANSWERS_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| Path::new(crate::PUZZLE_DIR).join("answers"));

pub(crate) fn create_answers_dir() -> Result<()> {
    fs::create_dir_all(&*PUZZLE_ANSWERS_DIR).with_context(|| {
        format!(
            "failed to create puzzle answers directory: {}",
            PUZZLE_ANSWERS_DIR.display()
        )
    })
}

const CRC: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_BZIP2);

pub(crate) struct DayAnswersCommit {
    pub(crate) answer1: String,
    pub(crate) answer2: String,
    pub(crate) answer1_checksum: u32,
    pub(crate) answer2_checksum: u32,
}

impl DayAnswersCommit {
    pub(crate) fn new(answer1: &dyn fmt::Display, answer2: &dyn fmt::Display) -> Self {
        let answer1 = answer1.to_string();
        let answer2 = answer2.to_string();

        let answer1_checksum = CRC.checksum(answer1.trim().as_bytes());
        let answer2_checksum = CRC.checksum(answer2.trim().as_bytes());

        Self {
            answer1,
            answer2,
            answer1_checksum,
            answer2_checksum,
        }
    }

    fn parse(commit_str: &str) -> Result<Self> {
        let mut parts = commit_str.splitn(3, "\n---\n");

        let (Some(checksums), Some(answer1), Some(answer2)) =
            (parts.next(), parts.next(), parts.next())
        else {
            bail!("invalid commit");
        };

        let (answer1_checksum, answer2_checksum) = match checksums.split_once(':') {
            Some((c1, c2)) => (c1.parse::<u32>()?, c2.parse::<u32>()?),
            None => bail!("invalid commit"),
        };

        Ok(Self {
            answer1: answer1.to_string(),
            answer2: answer2.to_string(),
            answer1_checksum,
            answer2_checksum,
        })
    }
}

impl PartialEq for DayAnswersCommit {
    fn eq(&self, other: &Self) -> bool {
        self.answer1_checksum == other.answer1_checksum
            && self.answer2_checksum == other.answer2_checksum
    }
}

impl fmt::Display for DayAnswersCommit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let answer1_checksum = self.answer1_checksum;
        let answer2_checksum = self.answer2_checksum;
        let answer1 = self.answer1.trim();
        let answer2 = self.answer2.trim();
        writeln!(f, "{answer1_checksum}:{answer2_checksum}")?;
        writeln!(f, "---")?;
        writeln!(f, "{answer1}")?;
        writeln!(f, "---")?;
        writeln!(f, "{answer2}")?;
        Ok(())
    }
}

pub(crate) fn get_existing_commit(day_i: u32) -> Result<Option<DayAnswersCommit>> {
    let commit_path = PUZZLE_ANSWERS_DIR.join(format!("day{day_i}"));
    let failed_to_parse = || {
        format!(
            "failed to parse checksum from existing commit file: {}",
            commit_path.display()
        )
    };
    match fs::read_to_string(&commit_path) {
        Ok(commit) => Ok(Some(
            DayAnswersCommit::parse(&commit).with_context(failed_to_parse)?,
        )),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| {
            format!(
                "failed to read existing commit file: {}",
                commit_path.display()
            )
        }),
    }
}

pub(crate) fn write_day_answers(day_i: u32, commit: &DayAnswersCommit) -> Result<()> {
    create_answers_dir()?;

    let commit_path = PUZZLE_ANSWERS_DIR.join(format!("day{day_i}"));
    let commit = commit.to_string();

    fs::write(&commit_path, commit).with_context(|| {
        format!(
            "failed to write puzzle commit to file: {}",
            commit_path.display()
        )
    })
}

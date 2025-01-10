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

#[derive(PartialEq)]
pub(crate) struct DayCommit {
    pub(crate) part1: DayPartCommit,
    pub(crate) part2: DayPartCommit,
}

impl DayCommit {
    pub(crate) fn new(answer1: &dyn fmt::Display, answer2: &dyn fmt::Display) -> Self {
        Self {
            part1: DayPartCommit::new(answer1),
            part2: DayPartCommit::new(answer2),
        }
    }

    fn parse(commit_str: &str) -> Result<Self> {
        let mut parts = commit_str.splitn(3, "\n---\n");

        let (Some(checksums), Some(answer1), Some(answer2)) =
            (parts.next(), parts.next(), parts.next())
        else {
            bail!("invalid commit");
        };

        let (checksum1, checksum2) = match checksums.split_once(':') {
            Some((c1, c2)) => (c1.parse::<u32>()?, c2.parse::<u32>()?),
            None => bail!("invalid commit"),
        };

        Ok(Self {
            part1: DayPartCommit {
                answer: answer1.to_string(),
                checksum: checksum1,
            },
            part2: DayPartCommit {
                answer: answer2.to_string(),
                checksum: checksum2,
            },
        })
    }
}

impl fmt::Display for DayCommit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let answer1_checksum = self.part1.checksum;
        let answer2_checksum = self.part2.checksum;
        let answer1 = self.part1.answer.trim();
        let answer2 = self.part2.answer.trim();
        writeln!(f, "{answer1_checksum}:{answer2_checksum}")?;
        writeln!(f, "---")?;
        writeln!(f, "{answer1}")?;
        writeln!(f, "---")?;
        writeln!(f, "{answer2}")?;
        Ok(())
    }
}

pub(crate) struct DayPartCommit {
    pub(crate) answer: String,
    pub(crate) checksum: u32,
}

impl PartialEq for DayPartCommit {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.checksum, &other.checksum)
    }
}

impl DayPartCommit {
    pub(crate) fn new(answer: &dyn fmt::Display) -> Self {
        let answer = answer.to_string();
        let checksum = CRC.checksum(answer.trim().as_bytes());
        Self { answer, checksum }
    }
}

pub(crate) fn get_existing_commit(day_i: u32) -> Result<Option<DayCommit>> {
    let commit_path = PUZZLE_ANSWERS_DIR.join(format!("day{day_i}"));
    let failed_to_parse = || {
        format!(
            "failed to parse checksum from existing commit file: {}",
            commit_path.display()
        )
    };
    match fs::read_to_string(&commit_path) {
        Ok(commit) => Ok(Some(
            DayCommit::parse(&commit).with_context(failed_to_parse)?,
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

pub(crate) fn write_day_answers(day_i: u32, commit: &DayCommit) -> Result<()> {
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

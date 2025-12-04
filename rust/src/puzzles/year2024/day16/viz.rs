use std::io::{self, Write};

use anyhow::Result;
use crossterm::{
    cursor, execute, queue,
    style::{self, Stylize},
    terminal,
};

mod dijkstras;

pub(super) use dijkstras::run as run_dijkstras;

pub(super) const SEP: &str = " / ";

pub(super) fn run<F, R>(imp: F) -> R
where
    F: FnOnce(&mut io::Stdout) -> Result<R>,
{
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).unwrap();
    terminal::enable_raw_mode().unwrap();

    let result = imp(&mut stdout);

    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show).unwrap();
    terminal::disable_raw_mode().unwrap();

    result.unwrap()
}

pub(super) fn draw_maze(stdout: &mut io::Stdout, rows: &[&str]) -> Result<()> {
    queue!(stdout, cursor::MoveTo(0, 0))?;

    for &row in rows {
        queue!(stdout, terminal::Clear(terminal::ClearType::CurrentLine))?;
        for b in row.bytes() {
            let content = match b {
                b'#' => "█".grey(),
                b'.' => " ".stylize(),
                b'S' => "S".bold().yellow(),
                b'E' => "E".bold().yellow(),
                _ => unreachable!(),
            };
            queue!(stdout, style::PrintStyledContent(content))?;
        }
        queue!(stdout, cursor::MoveToNextLine(1))?;
    }

    Ok(())
}

pub(super) fn draw_open_set(stdout: &mut io::Stdout, open_set: &[super::ScoredNode]) -> Result<()> {
    for (i, ns) in open_set.iter().enumerate() {
        queue!(
            stdout,
            cursor::MoveTo(ns.node.pos.col as u16, ns.node.pos.row as u16),
        )?;
        let mut content = match ns.node.dir {
            super::Direction::North => "^",
            super::Direction::East => ">",
            super::Direction::South => "v",
            super::Direction::West => "<",
        }
        .yellow();
        if i == 0 {
            content = content.on_yellow().black();
        }
        queue!(stdout, style::PrintStyledContent(content))?;
    }

    Ok(())
}

pub(super) fn draw_logs(stdout: &mut io::Stdout, logs: &[String]) -> Result<()> {
    stdout.flush()?;

    queue!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown))?;

    let (_cur_col, cur_row) = cursor::position()?;

    let (term_width, term_height) = terminal::size()?;
    let rows_remaining = term_height.saturating_sub(cur_row + 1) as usize;

    let log_chunks = logs
        .iter()
        .rev()
        .flat_map(|line| {
            let Some(i) = line.find(SEP) else {
                return vec![line.as_str()].into_iter().rev();
            };

            let mut chunk_start = 0;
            let mut chunk_end = i;
            let mut chunks = vec![];

            while chunk_end < line.len() {
                match line[chunk_end + SEP.len()..].find(SEP) {
                    Some(i) => {
                        if chunk_end - chunk_start + SEP.len() + i <= term_width as usize {
                            chunk_end += SEP.len() + i;
                        } else {
                            chunks.push(&line[chunk_start..chunk_end]);
                            chunk_start = chunk_end;
                            chunk_end += SEP.len() + i;
                        }
                    }
                    None => {
                        if line.len() - chunk_start <= term_width as usize {
                            chunks.push(&line[chunk_start..]);
                        } else {
                            chunks.push(&line[chunk_start..chunk_end]);
                            chunks.push(&line[chunk_end..]);
                        }
                        break;
                    }
                }
            }

            chunks.into_iter().rev()
        })
        .take(rows_remaining)
        .collect::<Vec<_>>();

    for chunk in log_chunks.into_iter().rev() {
        queue!(stdout, cursor::MoveToNextLine(1), style::Print(chunk))?;
    }

    stdout.flush()?;

    Ok(())
}

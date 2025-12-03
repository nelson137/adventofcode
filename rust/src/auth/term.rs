use std::{
    io::{self, Read},
    iter,
};

use crossterm::{
    cursor, execute, style,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};

pub(super) fn read_token() -> io::Result<Option<String>> {
    let mut gh_pat_reader = TokenReader::new()?;
    gh_pat_reader.read()
}

struct TokenReader {
    stdout: io::Stdout,
    buf: Vec<u8>,
}

impl TokenReader {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self {
            stdout: io::stdout(),
            buf: Vec::with_capacity(128),
        })
    }

    fn draw_asterisks(&mut self) -> io::Result<()> {
        let asterisks = String::from_iter(iter::repeat_n('*', self.buf.len()));
        execute!(
            self.stdout,
            cursor::RestorePosition,
            terminal::Clear(terminal::ClearType::UntilNewLine),
            style::Print(asterisks)
        )?;
        Ok(())
    }

    fn read(&mut self) -> io::Result<Option<String>> {
        execute!(
            self.stdout,
            style::Print("Session Token: "),
            cursor::SavePosition
        )?;

        let mut inbuf = [0_u8];

        while io::stdin().read(&mut inbuf).expect("read from stdin") == 1 {
            // execute!(self.stdout, style::Print(inbuf[0]), style::Print("\r\n"))?;
            match inbuf[0] {
                0x03 /* ^C */ | 0x04 /* ^D */ | 0x1b /* <Esc> */ => return Ok(None),
                0x0d /* \r */ => break,
                0x7f /* <BS> */ => {
                    self.buf.pop();
                    self.draw_asterisks()?;
                    // execute!(self.stdout, style::Print("<BS>\r\n"))?;
                }
                0x17 /* ^W */ | 0x08 /* ^<BS> */ => {
                    self.buf.clear();
                    self.draw_asterisks()?;
                    // execute!(self.stdout, style::Print("<clear>\r\n"))?;
                }
                c if c.is_ascii_graphic() || c.is_ascii_whitespace() => {
                    self.buf.push(c);
                    self.draw_asterisks()?;
                    // execute!(self.stdout, style::Print(format!("{:?}\r\n", c as char)))?;
                }
                _ => {}
            };
        }

        execute!(self.stdout, style::Print("\r\n"))?;

        let gh_pat = self
            .buf
            .iter()
            .copied()
            .filter(|b| b.is_ascii_graphic() && !b.is_ascii_whitespace())
            .map(|c| c as char)
            .collect::<String>();

        Ok(Some(gh_pat))
    }
}

impl Drop for TokenReader {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}

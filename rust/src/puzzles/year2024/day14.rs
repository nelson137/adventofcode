use std::{
    cmp,
    collections::HashMap,
    fmt,
    io::{self, Read, Write},
};

use anyhow::Result;
use crossterm::{cursor, execute, queue, style, terminal};
use nalgebra::Vector2;

inventory::submit!(
    crate::days::DayModule::new("2024", 14)
        .with_executors(
            crate::day_part_executors![part1],
            crate::day_part_executors![part2],
        )
        .with_pt2_visualizer(part2_viz)
);

struct Map {
    width: i64,
    height: i64,
    robots: Vec<Robot>,
}

impl Map {
    fn parse(input: &str) -> Self {
        let is_test = input.starts_with("TEST");
        let (width, height) = if is_test { (11, 7) } else { (101, 103) };

        let robots = input
            .lines()
            .skip(if is_test { 1 } else { 0 })
            .map(|line| {
                let mut parts = line.splitn(6, ['=', ',', ' ']);
                _ = parts.next().unwrap();
                let px = parts.next().unwrap().parse::<i64>().unwrap();
                let py = parts.next().unwrap().parse::<i64>().unwrap();
                _ = parts.next().unwrap();
                let vx = parts.next().unwrap().parse::<i64>().unwrap();
                let vy = parts.next().unwrap().parse::<i64>().unwrap();
                Robot::new((px, py), (vx, vy))
            })
            .collect();

        Self {
            width,
            height,
            robots,
        }
    }

    fn step(&mut self, n: u32) {
        for robot in &mut self.robots {
            robot.step(n, self.width, self.height);
        }
    }

    fn step_rev(&mut self, n: u32) {
        for robot in &mut self.robots {
            robot.step_rev(n, self.width, self.height);
        }
    }

    fn calculate_safety_factor(&self) -> u64 {
        let (mut q1_factor, mut q2_factor, mut q3_factor, mut q4_factor) = (0, 0, 0, 0);

        for robot in &self.robots {
            match robot.position.y.cmp(&(self.height / 2)) {
                cmp::Ordering::Less => match robot.position.x.cmp(&(self.width / 2)) {
                    cmp::Ordering::Greater => q1_factor += 1,
                    cmp::Ordering::Less => q2_factor += 1,
                    _ => {}
                },
                cmp::Ordering::Greater => match robot.position.x.cmp(&(self.width / 2)) {
                    cmp::Ordering::Greater => q4_factor += 1,
                    cmp::Ordering::Less => q3_factor += 1,
                    _ => {}
                },
                _ => {}
            }
        }

        q1_factor * q2_factor * q3_factor * q4_factor
    }

    fn find_easter_egg(&mut self) -> u32 {
        let mut robot_positions =
            vec![Vec::<u8>::with_capacity(self.robots.len() / 4); self.height as usize];

        fn has_contiguous_run(n: u32, values: &mut [u8]) -> bool {
            if values.len() < n as usize {
                return false;
            }

            values.sort_unstable();

            let mut run_x = values[0];
            let mut run_size = 1_u32;

            for &x in &values[1..] {
                if x == run_x + 1 {
                    run_size += 1;
                    if run_size >= n {
                        return true;
                    }
                } else {
                    run_size = 1;
                }
                run_x = x;
            }

            false
        }

        let mut steps = 0_u32;

        'find: loop {
            for robot in &self.robots {
                robot_positions[robot.position.y as usize].push(robot.position.x as u8);
            }

            for positions in &mut robot_positions {
                if has_contiguous_run(31, positions) {
                    break 'find;
                }
            }

            steps += 1;
            self.step(1);
            for positions in &mut robot_positions {
                positions.clear();
            }
        }

        steps
    }

    fn viz2_find_easter_egg(&mut self) -> Option<u32> {
        let mut stdout = io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).unwrap();
        terminal::enable_raw_mode().unwrap();

        let answer = self._viz2_find_easter_egg_impl(&mut stdout);

        execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show).unwrap();
        terminal::disable_raw_mode().unwrap();

        answer.unwrap()
    }

    fn _viz2_find_easter_egg_impl(&mut self, stdout: &mut io::Stdout) -> Result<Option<u32>> {
        let mut steps = 7344;
        let mut inbuf = [0, 0, 0, 0];
        let mut n_read;

        let mut guards = vec![0_u64; (self.width * (self.height + 1) / 2) as usize];

        self.step(steps);
        self._viz2_ee_draw(stdout, steps, &mut guards)?;

        loop {
            n_read = io::stdin().read(&mut inbuf)?;

            match (n_read, inbuf[0]) {
                // Escape | ^C | ^D | q | Q
                (1, 0x1b | 0x03 | 0x04 | b'q' | b'Q') => break Ok(None),
                // \r
                (1, 0x0d) => break Ok(Some(steps)),
                // Space
                (1, b' ') => {
                    steps += 1;
                    self.step(1);
                    self._viz2_ee_draw(stdout, steps, &mut guards)?;
                }
                // Right Arrow
                (3, 0x1b) if inbuf[1] == 0x5b && inbuf[2] == 0x43 => {
                    steps += 1;
                    self.step(1);
                    self._viz2_ee_draw(stdout, steps, &mut guards)?;
                }
                // n
                (1, b'n') => {
                    steps += 100;
                    self.step(100);
                    self._viz2_ee_draw(stdout, steps, &mut guards)?;
                }
                // N
                (1, b'N') => {
                    steps += 1000;
                    self.step(1000);
                    self._viz2_ee_draw(stdout, steps, &mut guards)?;
                }
                // Backspace
                (1, 0x7f) => {
                    if steps > 0 {
                        steps -= 1;
                        self.step_rev(1);
                    }
                    self._viz2_ee_draw(stdout, steps, &mut guards)?;
                }
                // Left Arrow
                (3, 0x1b) if inbuf[1] == 0x5b && inbuf[2] == 0x44 => {
                    if steps > 0 {
                        steps -= 1;
                        self.step_rev(1);
                    }
                    self._viz2_ee_draw(stdout, steps, &mut guards)?;
                }
                // b
                (1, b'b') => {
                    if steps > 100 {
                        steps -= 100;
                        self.step_rev(100);
                    }
                    self._viz2_ee_draw(stdout, steps, &mut guards)?;
                }
                // B
                (1, b'B') => {
                    if steps > 1000 {
                        steps -= 1000;
                        self.step_rev(1000);
                    }
                    self._viz2_ee_draw(stdout, steps, &mut guards)?;
                }
                _ => {}
            }
        }
    }

    fn _viz2_ee_draw(
        &mut self,
        stdout: &mut io::Stdout,
        steps: u32,
        robot_map: &mut [u64],
    ) -> Result<()> {
        let size = terminal::size()?;

        let move_to_steps = {
            let steps = steps.to_string();
            let steps_width = steps.len().min(u16::MAX as usize).max(4);
            cursor::MoveTo(size.0 - steps_width as u16, 0)
        };
        execute!(
            stdout,
            move_to_steps,
            style::Print(steps),
            terminal::Clear(terminal::ClearType::UntilNewLine)
        )?;

        const BORDER_TL: char = '╭';
        const BORDER_TR: char = '╮';
        const BORDER_BR: char = '╯';
        const BORDER_BL: char = '╰';
        const BORDER_VERT: char = '│';
        const BORDER_HOR: char = '─';

        queue!(stdout, cursor::MoveTo(0, 0), style::Print(BORDER_TL))?;
        for _ in 0..self.width {
            queue!(stdout, style::Print(BORDER_HOR))?;
        }
        queue!(stdout, style::Print(BORDER_TR))?;

        for _ in 0..(self.height + 1) / 2 {
            queue!(
                stdout,
                cursor::MoveToNextLine(1),
                style::Print(BORDER_VERT),
                cursor::MoveRight(self.width as u16),
                style::Print(BORDER_VERT)
            )?;
        }

        queue!(stdout, cursor::MoveToNextLine(1), style::Print(BORDER_BL))?;
        for _ in 0..self.width {
            queue!(stdout, style::Print(BORDER_HOR))?;
        }
        queue!(stdout, style::Print(BORDER_BR))?;

        const LOWER_FILLED: u64 = u32::MAX as u64;
        const UPPER_FILLED: u64 = (u32::MAX as u64) << 32;
        const BOTH_FILLED: u64 = LOWER_FILLED | UPPER_FILLED;

        robot_map.fill(0);

        for robot in &self.robots {
            let pos = robot.position;
            let map_y = pos.y / 2;
            let i = map_y * self.width + pos.x;
            robot_map[i as usize] |= if pos.y % 2 == 0 {
                LOWER_FILLED
            } else {
                UPPER_FILLED
            };
        }

        for (i, cell) in robot_map.iter().copied().enumerate() {
            let c = if cell == LOWER_FILLED {
                '▀'
            } else if cell == UPPER_FILLED {
                '▄'
            } else if cell == BOTH_FILLED {
                '█'
            } else {
                ' '
            };
            queue!(
                stdout,
                cursor::MoveTo(
                    (i % self.width as usize) as u16 + 1,
                    (i / self.width as usize) as u16 + 1,
                ),
                style::Print(c),
            )?;
        }

        stdout.flush()?;

        Ok(())
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut counts = HashMap::<Vec2, u32>::with_capacity(self.robots.len());
        for r in &self.robots {
            *counts.entry(r.position).or_default() += 1;
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let c = counts
                    .get(&Vec2::new(x, y))
                    .copied()
                    .map(|c| {
                        assert!(c <= u8::MAX as u32);
                        (b'0' + c as u8) as char
                    })
                    .unwrap_or('.');
                write!(f, "{c}")?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

type Vec2 = Vector2<i64>;

struct Robot {
    position: Vec2,
    velocity: Vec2,
}

impl Robot {
    fn new(pos: (i64, i64), vel: (i64, i64)) -> Self {
        Self {
            position: Vec2::new(pos.0, pos.1),
            velocity: Vec2::new(vel.0, vel.1),
        }
    }

    fn step(&mut self, n: u32, width: i64, height: i64) {
        self.position += n as i64 * self.velocity;
        self.fix(width, height);
    }

    fn step_rev(&mut self, n: u32, width: i64, height: i64) {
        self.position -= n as i64 * self.velocity;
        self.fix(width, height);
    }

    fn fix(&mut self, width: i64, height: i64) {
        self.position.x = ((self.position.x % width) + width) % width;
        self.position.y = ((self.position.y % height) + height) % height;
    }
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut map = Map::parse(input);
    // println!("{map}");

    map.step(100);
    // println!("{map}");

    let safety_factor = map.calculate_safety_factor();

    Some(Box::new(safety_factor))
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut map = Map::parse(input);

    let steps = map.find_easter_egg();

    // NOTE: The conditions for this solution (31 contiguous robots in a row)
    //       was found with the part 2 visualizer.
    assert_eq!(7344, steps);

    Some(Box::new(steps))
}

fn part2_viz(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut map = Map::parse(input);
    let steps = map.viz2_find_easter_egg();
    steps.map(|s| Box::new(s) as Box<dyn std::fmt::Display>)
}

#[cfg(test)]
mod tests {
    const SIZE: i64 = 20;

    const CASES_NEG: &[(i64, i64)] = &[
        (10, -110),
        (5, -115),
        (1, -119),
        (0, -120),
        (19, -121),
        (18, -122),
    ];

    const CASES_POS: &[(i64, i64)] = &[
        (10, 110),
        (15, 115),
        (19, 119),
        (0, 120),
        (1, 121),
        (2, 122),
    ];

    fn adjust_mult(mut x: i64) -> i64 {
        if x > SIZE {
            x -= SIZE * (x / SIZE)
        } else if x < 0 {
            x -= SIZE * ((x - SIZE + 1) / SIZE)
        }

        x
    }

    fn adjust_mod(x: i64) -> i64 {
        ((x % SIZE) + SIZE) % SIZE
    }

    #[test]
    fn adjust_neg() {
        for &(expected, x) in CASES_NEG {
            assert_eq!(expected, adjust_mult(x), "mult: {x}");
            assert_eq!(expected, adjust_mod(x), "mod: {x}");
        }
    }

    #[test]
    fn adjust_pos() {
        for &(expected, x) in CASES_POS {
            assert_eq!(expected, adjust_mult(x), "mult: {x}");
            assert_eq!(expected, adjust_mod(x), "mod: {x}");
        }
    }
}

use std::{cmp, collections::HashMap, fmt};

use nalgebra::Vector2;

crate::day_executors! {
    [part1]
    [part2]
}

crate::day_visualizers! {
    []
    []
}

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
    _ = input;

    None
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

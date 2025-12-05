inventory::submit!(crate::days::DayModule::new("2025", 1).with_executors(
    crate::day_part_executors![part1],
    crate::day_part_executors![part2],
));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Dial(i32);

impl Default for Dial {
    fn default() -> Self {
        Self(50)
    }
}

impl Dial {
    const MAX: i32 = 100;

    fn right(self, amount: RotationAmount) -> Self {
        let amount = amount as i32;
        let value = self.0 + amount;
        Self(value % Self::MAX)
    }

    fn left(self, amount: RotationAmount) -> Self {
        let amount = amount as i32 % Self::MAX;
        let value = self.0 + Self::MAX - amount;
        Self(value % Self::MAX)
    }
}

type RotationAmount = u16;

enum Rotation {
    Left(RotationAmount),
    Right(RotationAmount),
}

fn parse_rotations(input: &str) -> impl Iterator<Item = Rotation> {
    input.lines().map(|l| {
        let (direction, amount) = l.split_at(1);
        let amount = amount.parse().unwrap();
        match direction {
            "L" => Rotation::Left(amount),
            _ => Rotation::Right(amount),
        }
    })
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut dial = Dial::default();
    let mut times_wrapped = 0;

    for rotation in parse_rotations(input) {
        dial = match rotation {
            Rotation::Right(amount) => dial.right(amount),
            Rotation::Left(amount) => dial.left(amount),
        };
        if dial.0 == 0 {
            times_wrapped += 1;
        }
    }

    Some(Box::new(times_wrapped))
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    _ = input;

    None
}

#[cfg(test)]
mod tests {
    use std::iter;

    use super::*;

    #[test]
    fn dial_right() {
        let cases = iter::repeat(0..100_i32).flatten().take(250).enumerate();
        for (x, expected) in cases {
            assert_eq!(Dial(expected), Dial(0).right(x as u16));
        }
    }

    #[test]
    fn dial_left() {
        let cases = iter::once(0)
            .chain(iter::repeat((0..100_i32).rev()).flatten().take(249))
            .enumerate();
        for (x, expected) in cases {
            assert_eq!(Dial(expected), Dial(0).left(x as u16));
        }
    }
}

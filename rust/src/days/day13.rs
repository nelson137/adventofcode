use std::fmt;

crate::day_executors! {
    [part1]
    [part2]
}

crate::day_visualizers! {
    []
    []
}

struct ClawMachine {
    a: Button,
    b: Button,
    prize: Prize,
}

impl fmt::Display for ClawMachine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (a_x, a_y) = self.a.into();
        let (b_x, b_y) = self.b.into();
        let (p_x, p_y) = self.prize.into();
        write!(
            f,
            "[A] (x={a_x}, y={a_y}) | [B] (x={b_x}, y={b_y}) | [Prize] (x={p_x}, y={p_y})"
        )
    }
}

impl ClawMachine {
    fn new(a: Vec2, b: Vec2, prize: Vec2) -> Self {
        Self {
            a: Button(a),
            b: Button(b),
            prize: Prize(prize),
        }
    }

    /// Solve for the variables `A` and `B` in the Claw Machine Equations.
    ///
    /// Return the number of tokens required to play the Claw Machine and
    /// position the claw over the prize (`3*A + B`) if a solution exists.
    ///
    /// Equations:
    ///
    /// For some given Claw Machine Button vectors `(a_x, a_y)` and
    /// `(b_x, b_y)` and a Prize vector `(p_x, p_y)` there may exist some
    /// combination of variables `A` and `B` such that `A*a_x + B*b_x = p_x`
    /// (Eq1) and `A*a_y + B*b_y = p_y` (Eq2).
    ///
    /// With these equations, we can isolate `A` and `B` to create a
    /// constant-time algorithm for computing the values of `A` and `B`.
    ///
    /// Isolate `B` (using Eq2):
    ///
    /// ```
    /// A*a_y + B*b_y = p_y
    /// B*b_y = p_y - A*a_y
    /// B = (p_y - A*a_y) / b_y
    /// ```
    ///
    /// Solve for `A` (using Eq1):
    ///
    /// ```
    /// A*a_x + B*b_x = p_x
    /// A*a_x + b_x*(p_y - A*a_y)/b_y = p_x
    /// A*a_x + b_x*p_y/b_y - b_x*A*a_y/b_y = p_x
    /// A*(a_x - b_x*a_y/b_y) = p_x - b_x*p_y/b_y
    /// A = (p_x - b_x*p_y/b_y)/(a_x - b_x*a_y/b_y)
    /// ```
    ///
    /// For the solution to be valid
    fn solve(&self) -> Option<u32> {
        let (a_x, a_y) = self.a.into();
        let (b_x, b_y) = self.b.into();
        let (p_x, p_y) = self.prize.into();

        let a = (p_x - b_x * p_y / b_y) / (a_x - b_x * a_y / b_y);
        let b = (p_y - a * a_y) / b_y;

        match (a.to_whole_number(), b.to_whole_number()) {
            (Some(a), Some(b)) => Some(3 * a + b),
            _ => None,
        }
    }
}

trait FloatToWholeNumber {
    type Target;
    fn to_whole_number(self) -> Option<Self::Target>;
}

impl FloatToWholeNumber for f32 {
    type Target = u32;
    #[inline(always)]
    fn to_whole_number(self) -> Option<Self::Target> {
        const SOLUTION_EPSILON: f32 = 1000. * f32::EPSILON;
        let mut f = self.fract();
        if f < SOLUTION_EPSILON {
            return Some(self as u32);
        }
        f = 1. - f;
        if f < SOLUTION_EPSILON {
            return Some(self as u32 + 1);
        }
        None
    }
}

#[derive(Clone, Copy)]
struct Button(Vec2);

impl From<Button> for (f32, f32) {
    #[inline(always)]
    fn from(val: Button) -> Self {
        val.0.into()
    }
}

#[derive(Clone, Copy)]
struct Prize(Vec2);

impl From<Prize> for (f32, f32) {
    #[inline(always)]
    fn from(val: Prize) -> Self {
        val.0.into()
    }
}

#[derive(Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    #[inline(always)]
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[inline(always)]
fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

impl From<Vec2> for (f32, f32) {
    #[inline(always)]
    fn from(val: Vec2) -> Self {
        (val.x, val.y)
    }
}

fn parse(input: &str) -> Vec<ClawMachine> {
    let mut claw_machines = Vec::new();

    #[inline(always)]
    fn parse_claw_machine(
        (a_x, a_y): (&str, &str),
        (b_x, b_y): (&str, &str),
        prize: &str,
    ) -> ClawMachine {
        let a = vec2(a_x.parse().unwrap(), a_y.parse().unwrap());

        let b = vec2(b_x.parse().unwrap(), b_y.parse().unwrap());

        let (_, prize_values) = prize.split_once(": ").unwrap();
        let (p_x, p_y) = prize_values.split_once(", ").unwrap();
        let (p_x, p_y) = (&p_x[2..], &p_y[2..]);

        let prize = vec2(p_x.parse().unwrap(), p_y.parse().unwrap());

        ClawMachine::new(a, b, prize)
    }

    let mut claw_machine_line = 0_u8;
    let (mut a_x, mut a_y) = ("", "");
    let (mut b_x, mut b_y) = ("", "");
    let mut prize_line = "";

    for line in input.lines() {
        match claw_machine_line {
            0 => (a_x, a_y) = (&line[12..14], &line[18..20]),
            1 => (b_x, b_y) = (&line[12..14], &line[18..20]),
            2 => prize_line = line,
            _ => {
                claw_machines.push(parse_claw_machine((a_x, a_y), (b_x, b_y), prize_line));
                claw_machine_line = 0;
                continue;
            }
        }
        claw_machine_line += 1;
    }

    claw_machines.push(parse_claw_machine((a_x, a_y), (b_x, b_y), prize_line));

    claw_machines
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let claw_machines = parse(input);

    let answer = claw_machines
        .iter()
        .filter_map(ClawMachine::solve)
        .sum::<u32>();

    Some(Box::new(answer))
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    _ = input;

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const CASES: &[(Option<u32>, f32)] = &[
        (Some(0), 0.),
        (Some(0), 0. + f32::EPSILON),
        (None, 0.2),
        (None, 0.8),
        (Some(1), 1. - f32::EPSILON),
        (Some(1), 1.),
        (Some(1), 1. + f32::EPSILON),
        (None, 1.2),
    ];

    #[test]
    fn to_whole_number() {
        for &(expected, x) in CASES {
            assert_eq!(expected, x.to_whole_number());
        }
    }
}

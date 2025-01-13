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

impl ClawMachine {
    fn new(a: Vec2, b: Vec2, prize: Vec2) -> Self {
        Self {
            a: Button(a),
            b: Button(b),
            prize: Prize(prize),
        }
    }

    fn solve(&self) -> Option<u32> {
        let (a_x, a_y) = self.a.into();
        let (b_x, b_y) = self.b.into();
        let (p_x, p_y) = self.prize.into();

        for a in 1..=100 {
            for b in 1..=100 {
                let x = a * a_x + b * b_x;
                let y = a * a_y + b * b_y;
                if (x, y) == (p_x, p_y) {
                    return Some(3 * a + b);
                }
            }
        }

        None
    }
}

#[derive(Clone, Copy)]
struct Button(Vec2);

impl From<Button> for (u32, u32) {
    #[inline(always)]
    fn from(val: Button) -> Self {
        val.0.into()
    }
}

#[derive(Clone, Copy)]
struct Prize(Vec2);

impl From<Prize> for (u32, u32) {
    #[inline(always)]
    fn from(val: Prize) -> Self {
        val.0.into()
    }
}

#[derive(Clone, Copy)]
struct Vec2 {
    x: u32,
    y: u32,
}

impl Vec2 {
    #[inline(always)]
    fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[inline(always)]
fn vec2(x: u32, y: u32) -> Vec2 {
    Vec2::new(x, y)
}

impl From<Vec2> for (u32, u32) {
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

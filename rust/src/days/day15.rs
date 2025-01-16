use std::{
    fmt::{self, Write},
    ops,
};

use crossterm::style::Stylize;

crate::day_executors! {
    [part1]
    [part2]
}

crate::day_visualizers! {
    []
    []
}

fn parse(input: &str) -> (Map, Vec<Instruction>, Pos) {
    let width = input.lines().next().unwrap().trim().len();

    let mut warehouse = Vec::new();
    let mut robot = Pos::default();

    let mut lines = input.lines();
    let mut y = 0;

    while let Some(l) = lines.next().filter(|l| !l.is_empty()) {
        for (x, b) in l.bytes().enumerate() {
            match b {
                b'#' => warehouse.push(Cell::Wall),
                b'O' => warehouse.push(Cell::Box),
                b'.' => warehouse.push(Cell::Empty),
                b'@' => {
                    warehouse.push(Cell::Empty);
                    robot = Pos::new(x, y);
                }
                _ => unreachable!(),
            }
        }
        y += 1;
    }

    let robot_instructions = lines
        .flat_map(str::bytes)
        .map(Instruction::from)
        .collect::<Vec<_>>();

    (Map::new(width, warehouse), robot_instructions, robot)
}

struct Map {
    width: usize,
    warehouse: Vec<Cell>,
}

impl Map {
    fn new(width: usize, warehouse: Vec<Cell>) -> Self {
        Self { width, warehouse }
    }

    #[allow(dead_code)]
    fn print_warehouse(&self, robot: Pos) {
        let robot = robot.index(self.width);
        for (i, c) in self.warehouse.iter().copied().enumerate() {
            if i % self.width == 0 && i > 0 {
                println!();
            }
            if i == robot {
                print!("{}", '@'.bold().yellow());
            } else {
                print!("{c}");
            }
        }
        println!();
    }

    fn run_robot(&mut self, instructions: &[Instruction], mut robot: Pos) {
        for ins in instructions.iter().copied() {
            let probe_start = robot.move_(ins);
            let mut probe = probe_start;

            match self[probe] {
                Cell::Wall => {}
                Cell::Empty => robot = probe,
                Cell::Box => loop {
                    probe = probe.move_(ins);
                    match self[probe] {
                        Cell::Wall => break,
                        Cell::Box => {}
                        Cell::Empty => {
                            self.warehouse
                                .swap(probe_start.index(self.width), probe.index(self.width));
                            robot = probe_start;
                            break;
                        }
                    }
                },
            }
        }
    }

    fn sum_box_coords(&self) -> u64 {
        self.warehouse
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, c)| matches!(c, Cell::Box))
            .map(|(i, _)| Pos::from_index(i, self.width).gps_coord())
            .sum()
    }
}

#[derive(Clone, Copy, Default)]
struct Pos {
    y: u32,
    x: u32,
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},{}]", self.y, self.x)
    }
}

impl Pos {
    fn new(row: usize, col: usize) -> Self {
        Self {
            y: row as u32,
            x: col as u32,
        }
    }

    fn from_index(index: usize, width: usize) -> Self {
        Self {
            y: (index / width) as u32,
            x: (index % width) as u32,
        }
    }

    fn index(self, width: usize) -> usize {
        self.y as usize * width + self.x as usize
    }

    fn gps_coord(self) -> u64 {
        (100 * self.y + self.x) as u64
    }

    #[allow(dead_code)]
    fn nn(self) -> Self {
        Self {
            y: self.y - 1,
            x: self.x,
        }
    }

    #[allow(dead_code)]
    fn ee(self) -> Self {
        Self {
            y: self.y,
            x: self.x + 1,
        }
    }

    #[allow(dead_code)]
    fn ss(self) -> Self {
        Self {
            y: self.y + 1,
            x: self.x,
        }
    }

    #[allow(dead_code)]
    fn ww(self) -> Self {
        Self {
            y: self.y,
            x: self.x - 1,
        }
    }

    fn move_(self, ins: Instruction) -> Self {
        match ins {
            Instruction::North => self.nn(),
            Instruction::East => self.ee(),
            Instruction::South => self.ss(),
            Instruction::West => self.ww(),
        }
    }
}

impl ops::Index<Pos> for Map {
    type Output = Cell;

    #[inline(always)]
    fn index(&self, pos: Pos) -> &Self::Output {
        &self.warehouse[pos.index(self.width)]
    }
}

impl ops::IndexMut<Pos> for Map {
    #[inline(always)]
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        &mut self.warehouse[pos.index(self.width)]
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
#[allow(dead_code)]
enum Instruction {
    North = b'^',
    East = b'>',
    South = b'v',
    West = b'<',
}

impl From<u8> for Instruction {
    #[inline(always)]
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self as u8 as char)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Wall,
    Box,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Wall => f.write_char('#'),
            Self::Box => f.write_char('O'),
            Self::Empty => f.write_char('.'),
        }
    }
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, instructions, robot) = parse(input);

    map.run_robot(&instructions, robot);

    let answer = map.sum_box_coords();

    Some(Box::new(answer))
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    _ = input;

    None
}

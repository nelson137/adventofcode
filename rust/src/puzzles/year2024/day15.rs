use std::{
    fmt::{self, Write},
    ops,
};

use crossterm::style::Stylize;

inventory::submit!(crate::days::DayModule::new(2024, 15).with_executors(
    crate::day_part_executors![part1],
    crate::day_part_executors![part2],
));

fn parse_v1(input: &str) -> (Map, Vec<Instruction>, Pos) {
    let width = input.lines().next().unwrap().trim().len();

    let mut warehouse = Vec::new();
    let mut robot = Pos::default();

    let mut lines = input.lines();
    let mut y = 0;

    while let Some(l) = lines.next().filter(|l| !l.is_empty()) {
        for (x, b) in l.bytes().enumerate() {
            match b {
                b'#' => warehouse.push(Cell::Wall),
                b'O' => warehouse.push(Cell::BOX_SINGLE),
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

fn parse_v2(input: &str) -> (Map, Vec<Instruction>, Pos) {
    let width = 2 * input.lines().next().unwrap().trim().len();

    let mut warehouse = Vec::new();
    let mut robot = Pos::default();

    let mut lines = input.lines();
    let mut y = 0;

    while let Some(l) = lines.next().filter(|l| !l.is_empty()) {
        for (x, b) in l.bytes().enumerate() {
            match b {
                b'#' => warehouse.extend([Cell::Wall, Cell::Wall]),
                b'O' => warehouse.extend([Cell::BOX_LEFT, Cell::BOX_RIGHT]),
                b'.' => warehouse.extend([Cell::Empty, Cell::Empty]),
                b'@' => {
                    warehouse.extend([Cell::Empty, Cell::Empty]);
                    robot = Pos::new(2 * x, y);
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

#[allow(dead_code)]
fn read_line() {
    std::io::stdin().read_line(&mut String::new()).unwrap();
}

struct Map {
    width: usize,
    warehouse: Vec<Cell>,
}

impl Map {
    fn new(width: usize, warehouse: Vec<Cell>) -> Self {
        Self { width, warehouse }
    }

    #[inline(always)]
    fn index(&self, pos: Pos) -> usize {
        pos.index(self.width)
    }

    #[allow(dead_code)]
    fn print_warehouse(&self, robot: Pos) {
        let robot = self.index(robot);
        for (i, c) in self.warehouse.iter().copied().enumerate() {
            if i % self.width == 0 && i > 0 {
                println!();
            }
            if i == robot {
                print!("{}", '@'.bold().red());
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
                Cell::Box(_) => loop {
                    probe = probe.move_(ins);
                    match self[probe] {
                        Cell::Wall => break,
                        Cell::Box(_) => {}
                        Cell::Empty => {
                            let i = self.index(probe_start);
                            let j = self.index(probe);
                            self.warehouse.swap(i, j);
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
            .filter(|(_, c)| matches!(c, Cell::Box(BoxType::Single | BoxType::Left)))
            .map(|(i, _)| Pos::from_index(i, self.width).gps_coord())
            .sum()
    }

    #[inline(always)]
    fn wide_range(&self, pos: Pos) -> ops::Range<usize> {
        pos.wide_range(self.width)
    }

    #[inline(always)]
    fn get_wide(&self, pos: Pos) -> [Cell; 2] {
        let mut slot = <[Cell; 2]>::default();
        slot.copy_from_slice(&self.warehouse[self.wide_range(pos)]);
        slot
    }

    #[inline(always)]
    fn swap_wide(&mut self, a: Pos, b: Pos) {
        let b_range = self.wide_range(b);
        let a_i = self.index(a);
        // x <- a
        let x = self.get_wide(a);
        // a <- b
        self.warehouse.copy_within(b_range.clone(), a_i);
        // b <- x
        self.warehouse[b_range].copy_from_slice(&x);
    }

    fn run_robot_wide(&mut self, instructions: &[Instruction], mut robot: Pos) {
        let mut pushtree_gen = Vec::<Pos>::new();
        let mut pushtree_next_gen = Vec::<Pos>::new();
        let mut pushtree_swap_stack = Vec::<(Pos, Pos)>::new();

        for ins in instructions.iter().copied() {
            let probe = robot.move_(ins);
            match self[probe] {
                Cell::Wall => {}
                Cell::Empty => robot = probe,
                Cell::Box(boxt) => {
                    if let Some(next) = match ins {
                        Instruction::East | Instruction::West => {
                            self.move_boxes_wide_ew(probe, ins)
                        }
                        Instruction::North | Instruction::South => self.move_boxes_wide_ns(
                            probe,
                            ins,
                            boxt,
                            &mut pushtree_gen,
                            &mut pushtree_next_gen,
                            &mut pushtree_swap_stack,
                        ),
                    } {
                        robot = next;
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn move_boxes_wide_ew(&mut self, pos: Pos, ins: Instruction) -> Option<Pos> {
        let mut probe = pos;
        loop {
            probe = probe.move_(ins);
            match self[probe] {
                Cell::Wall => return None,
                Cell::Box(_) => {}
                Cell::Empty => {
                    let start_i = self.index(pos);
                    let end_i = self.index(probe);
                    let (range, dest_i) = match ins {
                        Instruction::East => (start_i..end_i, start_i + 1),
                        Instruction::West => (end_i + 1..start_i + 1, end_i),
                        _ => unreachable!(),
                    };
                    self.warehouse.copy_within(range, dest_i);
                    self.warehouse[start_i] = Cell::Empty;
                    return Some(pos);
                }
            }
        }
    }

    #[inline(always)]
    fn move_boxes_wide_ns(
        &mut self,
        pos: Pos,
        ins: Instruction,
        boxt: BoxType,
        pushtree_gen: &mut Vec<Pos>,
        pushtree_next_gen: &mut Vec<Pos>,
        pushtree_swap_stack: &mut Vec<(Pos, Pos)>,
    ) -> Option<Pos> {
        let seed = if boxt == BoxType::Right {
            pos.ww()
        } else {
            pos
        };

        pushtree_gen.clear();
        pushtree_next_gen.clear();
        pushtree_swap_stack.clear();
        pushtree_gen.push(seed);

        while !pushtree_gen.is_empty() {
            for box_pos in pushtree_gen.drain(..) {
                let next_pos = box_pos.move_(ins);
                match self.get_wide(next_pos) {
                    // Box tree is in contact with a wall
                    [Cell::Wall, _] | [_, Cell::Wall] => return None,
                    // Box can move
                    [Cell::Empty, Cell::Empty] => {
                        pushtree_swap_stack.push((box_pos, next_pos));
                    }
                    // There is another box directly in front of this one
                    [Cell::BOX_LEFT, Cell::BOX_RIGHT] => {
                        let next_box_pos = next_pos;
                        pushtree_swap_stack.push((box_pos, next_pos));
                        pushtree_next_gen.push(next_box_pos);
                    }
                    // There is a box in front of this one on the west side
                    [Cell::BOX_RIGHT, Cell::Empty] => {
                        let next_box_pos = next_pos.ww();
                        pushtree_swap_stack.push((box_pos, next_pos));
                        if pushtree_next_gen.last() != Some(&next_box_pos) {
                            pushtree_next_gen.push(next_box_pos);
                        }
                    }
                    // There is a box in front of this one on the east side
                    [Cell::Empty, Cell::BOX_LEFT] => {
                        let next_box_pos = next_pos.ee();
                        pushtree_swap_stack.push((box_pos, next_pos));
                        pushtree_next_gen.push(next_box_pos);
                    }
                    // There are two boxes in front of this one
                    [Cell::BOX_RIGHT, Cell::BOX_LEFT] => {
                        pushtree_swap_stack.push((box_pos, next_pos));

                        let next_box_pos = next_pos.ww();
                        if pushtree_next_gen.last() != Some(&next_box_pos) {
                            pushtree_next_gen.push(next_box_pos);
                        }

                        let next_box_pos = next_pos.ee();
                        pushtree_next_gen.push(next_box_pos);
                    }
                    // Invalid configurations
                    [Cell::BOX_SINGLE, _] | [_, Cell::BOX_SINGLE] => unreachable!(),
                    [Cell::BOX_LEFT, Cell::Empty] | [Cell::Empty, Cell::BOX_RIGHT] => {
                        unreachable!()
                    }
                    [Cell::BOX_LEFT, Cell::BOX_LEFT] => unreachable!(),
                    [Cell::BOX_RIGHT, Cell::BOX_RIGHT] => unreachable!(),
                }
            }

            std::mem::swap(pushtree_gen, pushtree_next_gen);
        }

        while let Some((a, b)) = pushtree_swap_stack.pop() {
            self.swap_wide(a, b);
        }

        Some(pos)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
struct Pos {
    x: u32,
    y: u32,
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},{}]", self.x, self.y)
    }
}

impl Pos {
    fn new(x: usize, y: usize) -> Self {
        Self {
            y: y as u32,
            x: x as u32,
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

    fn wide_range(&self, width: usize) -> ops::Range<usize> {
        let index = self.index(width);
        index..index + 2
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
        let i = self.index(pos);
        &self.warehouse[i]
    }
}

impl ops::IndexMut<Pos> for Map {
    #[inline(always)]
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        let i = self.index(pos);
        &mut self.warehouse[i]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
enum Instruction {
    North = b'^',
    South = b'v',
    East = b'>',
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

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Cell {
    #[default]
    Empty,
    Wall,
    Box(BoxType),
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Wall => f.write_char('#'),
            Self::BOX_SINGLE => f.write_char('O'),
            Self::BOX_LEFT => f.write_char('['),
            Self::BOX_RIGHT => f.write_char(']'),
            Self::Empty => f.write_char('.'),
        }
    }
}

impl Cell {
    const BOX_SINGLE: Self = Self::Box(BoxType::Single);
    const BOX_LEFT: Self = Self::Box(BoxType::Left);
    const BOX_RIGHT: Self = Self::Box(BoxType::Right);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BoxType {
    Single,
    Left,
    Right,
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, instructions, robot) = parse_v1(input);

    map.run_robot(&instructions, robot);

    let answer = map.sum_box_coords();

    Some(Box::new(answer))
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, instructions, robot) = parse_v2(input);

    map.run_robot_wide(&instructions, robot);

    let answer = map.sum_box_coords();

    Some(Box::new(answer))
}

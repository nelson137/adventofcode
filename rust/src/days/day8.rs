use std::{collections::HashMap, fmt, num::NonZeroU8, ops};

use itertools::Itertools;

crate::day_executors! {
    [part1]
    [part2]
}

crate::day_visualizers! {
    []
    []
}

fn parse(input: &str) -> Map {
    let height = input.lines().count();
    let width = input.lines().next().unwrap().len();
    let mut map = Map::new(height, width);

    for (r, line) in input.lines().enumerate() {
        for (c, b) in line.bytes().enumerate() {
            let pos = Pos::new(r, c);
            match b {
                b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' => {
                    // SAFETY: Due to the match it is not possible for `b` to be `0` which is the
                    //         only value for which `NonZeroU8::new` checks.
                    map.antennas[pos] = Some(Antenna(unsafe { NonZeroU8::new_unchecked(b) }))
                }
                b'.' | b'#' => {}
                _ => unreachable!(),
            }
        }
    }

    map
}

struct Map {
    height: usize,
    width: usize,
    antennas: AntennaGrid,
    antinodes: AntinodeGrid,
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.antennas.0 {
            for &cell in line {
                match cell {
                    Some(antenna) => write!(f, "{antenna}"),
                    None => write!(f, "."),
                }?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Map {
    fn new(height: usize, width: usize) -> Self {
        Self {
            height,
            width,
            antennas: AntennaGrid::new(height, width, None),
            antinodes: AntinodeGrid::new(height, width, false),
        }
    }

    fn contains_pos(&self, pos: Pos) -> bool {
        (0..self.height as i32).contains(&pos.row) && (0..self.width as i32).contains(&pos.col)
    }

    fn iter_antennas(&self) -> impl Iterator<Item = (Pos, Antenna)> {
        self.antennas
            .0
            .iter()
            .enumerate()
            .flat_map(|(r, line)| {
                line.iter()
                    .copied()
                    .enumerate()
                    .map(move |(c, cell)| (Pos::new(r, c), cell))
            })
            .filter_map(|(pos, cell)| cell.map(|this| (pos, this)))
    }

    fn find_antinodes(&mut self) -> usize {
        let mut freq_positions = HashMap::<Antenna, Vec<Pos>>::new();
        for (pos, antenna) in self.iter_antennas() {
            freq_positions.entry(antenna).or_default().push(pos);
        }

        for (_a, positions) in freq_positions {
            for pair in positions.into_iter().permutations(2) {
                let (a, b) = (pair[0], pair[1]);
                let delta = b - a;
                for antinode in [a - delta, b + delta] {
                    if self.contains_pos(antinode) {
                        self.antinodes[antinode] = true;
                    }
                }
            }
        }

        self.antinodes
            .0
            .iter()
            .flat_map(|row| row.iter().copied())
            .filter(|x| *x)
            .count()
    }
}

type AntennaGrid = Grid<Option<Antenna>>;
type AntinodeGrid = Grid<bool>;

struct Grid<T>(Vec<Vec<T>>);

impl<T: Clone> Grid<T> {
    fn new(height: usize, width: usize, value: T) -> Self {
        Self(vec![vec![value; width]; height])
    }
}

impl<T> ops::Index<Pos> for Grid<T> {
    type Output = T;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.0[index.row as usize][index.col as usize]
    }
}

impl<T> ops::IndexMut<Pos> for Grid<T> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.0[index.row as usize][index.col as usize]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Antenna(NonZeroU8);

impl fmt::Display for Antenna {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.get() as char)
    }
}

#[derive(Clone, Copy)]
struct Pos {
    row: i32,
    col: i32,
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},{}]", self.row, self.col)
    }
}

impl Pos {
    fn new(row: usize, col: usize) -> Self {
        Self {
            row: row as i32,
            col: col as i32,
        }
    }
}

impl ops::Add for Pos {
    type Output = Pos;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            row: self.row + rhs.row,
            col: self.col + rhs.col,
        }
    }
}

impl ops::Sub for Pos {
    type Output = Pos;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            row: self.row - rhs.row,
            col: self.col - rhs.col,
        }
    }
}

pub(super) fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut map = parse(input);

    let answer = map.find_antinodes();

    Some(Box::new(answer))
}

pub(super) fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    _ = input;

    None
}

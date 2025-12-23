use std::{collections::HashMap, fmt, num::NonZeroU8, ops};

inventory::submit!(crate::days::DayModule::new(2024, 8).with_executors(
    crate::day_part_executors![part1],
    crate::day_part_executors![part2],
));

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

    #[inline(always)]
    /// Set an antinode on the map.
    ///
    /// Returns whether the antinode was *newly set*, meaning that before this
    /// operation the cell at the given position `pos` was `false`.
    fn set_antinode(&mut self, pos: Pos) -> bool {
        let mut b = true;
        std::mem::swap(&mut self.antinodes[pos], &mut b);
        !b
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

        let mut antinode_count = 0;

        for (_a, positions) in freq_positions {
            for i in 0..positions.len() {
                for j in 0..positions.len() {
                    if i == j {
                        continue;
                    }
                    let (a, b) = (positions[i], positions[j]);
                    let delta = b - a;
                    for antinode in [a - delta, b + delta] {
                        if self.contains_pos(antinode) && self.set_antinode(antinode) {
                            antinode_count += 1;
                        }
                    }
                }
            }
        }

        antinode_count
    }

    fn find_antinodes_with_resonant_harmonics(&mut self) -> usize {
        let mut freq_positions = HashMap::<Antenna, Vec<Pos>>::new();

        let mut antinode_count = 0;

        for (pos, antenna) in self.iter_antennas() {
            freq_positions.entry(antenna).or_default().push(pos);
            antinode_count += 1;
        }

        for (_a, positions) in freq_positions {
            for i in 0..positions.len() {
                for j in 0..positions.len() {
                    if i == j {
                        continue;
                    }

                    let (a, b) = (positions[i], positions[j]);
                    let delta = b - a;

                    let mut antinode = a - delta;

                    while self.contains_pos(antinode) {
                        if self.set_antinode(antinode) && self.antennas[antinode].is_none() {
                            antinode_count += 1;
                        }
                        antinode -= delta;
                    }

                    let mut antinode = b + delta;

                    while self.contains_pos(antinode) {
                        if self.set_antinode(antinode) && self.antennas[antinode].is_none() {
                            antinode_count += 1;
                        }
                        antinode += delta;
                    }
                }
            }
        }

        antinode_count
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

impl ops::AddAssign for Pos {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
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

impl ops::SubAssign for Pos {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut map = parse(input);

    let answer = map.find_antinodes();

    Some(Box::new(answer))
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut map = parse(input);

    let answer = map.find_antinodes_with_resonant_harmonics();

    Some(Box::new(answer))
}

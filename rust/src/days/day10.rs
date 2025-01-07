use std::{collections::HashSet, ops};

crate::day_executors! {
    [part1]
    [part2]
}

crate::day_visualizers! {
    []
    []
}

struct Map {
    height: usize,
    width: usize,
    grid: Vec<Vec<u8>>,
}

impl Map {
    fn parse(input: &str) -> Self {
        let input = input.trim();
        let height = input.lines().count();
        let width = input.lines().next().unwrap().trim().len();
        let grid = input.lines().map(|line| line.bytes().collect()).collect();
        Self {
            height,
            width,
            grid,
        }
    }

    fn score_trailhead(&self, pos: Pos, trailends: &mut HashSet<Pos>) -> u32 {
        trailends.clear();
        self._score_trailhead_recurse(pos, b'0', trailends);
        trailends.len() as u32
    }

    fn _score_trailhead_recurse(&self, pos: Pos, height: u8, trailends: &mut HashSet<Pos>) {
        if height == b'9' {
            trailends.insert(pos);
            return;
        }

        let next = height + 1;

        if pos.col > 0 {
            let pos_next = pos.ww();
            if self[pos_next] == next {
                self._score_trailhead_recurse(pos_next, next, trailends);
            }
        }

        if pos.col < self.width as u32 - 1 {
            let pos_next = pos.ee();
            if self[pos_next] == next {
                self._score_trailhead_recurse(pos_next, next, trailends);
            }
        }

        if pos.row > 0 {
            let pos_next = pos.nn();
            if self[pos_next] == next {
                self._score_trailhead_recurse(pos_next, next, trailends);
            }
        }

        if pos.row < self.height as u32 - 1 {
            let pos_next = pos.ss();
            if self[pos_next] == next {
                self._score_trailhead_recurse(pos_next, next, trailends);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
struct Pos {
    row: u32,
    col: u32,
}

impl Pos {
    const fn new(row: usize, col: usize) -> Self {
        Self {
            row: row as u32,
            col: col as u32,
        }
    }

    fn nn(self) -> Self {
        Self {
            row: self.row - 1,
            col: self.col,
        }
    }

    fn ee(self) -> Self {
        Self {
            row: self.row,
            col: self.col + 1,
        }
    }

    fn ss(self) -> Self {
        Self {
            row: self.row + 1,
            col: self.col,
        }
    }

    fn ww(self) -> Self {
        Self {
            row: self.row,
            col: self.col - 1,
        }
    }
}

impl ops::Index<Pos> for Map {
    type Output = u8;

    #[inline(always)]
    fn index(&self, index: Pos) -> &Self::Output {
        &self.grid[index.row as usize][index.col as usize]
    }
}

pub(super) fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let map = Map::parse(input);

    let mut answer = 0;
    let mut trailends = HashSet::<Pos>::new();

    for (r, row) in map.grid.iter().enumerate() {
        for (c, cell) in row.iter().copied().enumerate() {
            if cell == b'0' {
                let pos = Pos::new(r, c);
                answer += map.score_trailhead(pos, &mut trailends);
            }
        }
    }

    Some(Box::new(answer))
}

pub(super) fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    _ = input;

    None
}

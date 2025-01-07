use std::{
    collections::{HashMap, HashSet},
    ops,
};

crate::day_executors! {
    [part1]
    [part2]
}

crate::day_visualizers! {
    []
    []
}

struct Map<'input> {
    height: usize,
    width: usize,
    grid: Vec<&'input [u8]>,
}

impl<'input> Map<'input> {
    fn parse(input: &'input str) -> Self {
        let height = input.lines().count();
        let width = input.lines().next().unwrap().trim().len();
        let mut grid = Vec::with_capacity(height);
        grid.extend(input.lines().map(|line| line.as_bytes()));
        Self {
            height,
            width,
            grid,
        }
    }

    fn iter_trailheads(&self) -> impl Iterator<Item = Pos> {
        self.grid.iter().copied().enumerate().flat_map(|(r, row)| {
            row.iter()
                .copied()
                .enumerate()
                .filter(|&(_, cell)| cell == b'0')
                .map(move |(c, _)| Pos::new(r, c))
        })
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

        #[inline(always)]
        fn _trail_step(map: &Map, pos_next: Pos, next: u8, trailends: &mut HashSet<Pos>) {
            if map[pos_next] == next {
                map._score_trailhead_recurse(pos_next, next, trailends);
            }
        }

        if pos.col > 0 {
            _trail_step(self, pos.ww(), next, trailends);
        }

        if pos.col < self.width as u32 - 1 {
            _trail_step(self, pos.ee(), next, trailends);
        }

        if pos.row > 0 {
            _trail_step(self, pos.nn(), next, trailends);
        }

        if pos.row < self.height as u32 - 1 {
            _trail_step(self, pos.ss(), next, trailends);
        }
    }

    fn rate_trailhead(&self, pos: Pos, trailends: &mut HashMap<Pos, u32>) -> u32 {
        trailends.clear();
        self._rate_trailhead_recurse(pos, b'0', trailends);
        trailends.values().sum()
    }

    fn _rate_trailhead_recurse(&self, pos: Pos, height: u8, trailends: &mut HashMap<Pos, u32>) {
        if height == b'9' {
            *trailends.entry(pos).or_default() += 1;
            return;
        }

        let next = height + 1;

        #[inline(always)]
        fn _trail_step(map: &Map, pos_next: Pos, next: u8, trailends: &mut HashMap<Pos, u32>) {
            if map[pos_next] == next {
                map._rate_trailhead_recurse(pos_next, next, trailends);
            }
        }

        if pos.col > 0 {
            _trail_step(self, pos.ww(), next, trailends);
        }

        if pos.col < self.width as u32 - 1 {
            _trail_step(self, pos.ee(), next, trailends);
        }

        if pos.row > 0 {
            _trail_step(self, pos.nn(), next, trailends);
        }

        if pos.row < self.height as u32 - 1 {
            _trail_step(self, pos.ss(), next, trailends);
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

impl ops::Index<Pos> for Map<'_> {
    type Output = u8;

    #[inline(always)]
    fn index(&self, index: Pos) -> &Self::Output {
        &self.grid[index.row as usize][index.col as usize]
    }
}

pub(super) fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let map = Map::parse(input);

    let mut trailends = HashSet::<Pos>::new();

    let answer = map
        .iter_trailheads()
        .map(|pos| map.score_trailhead(pos, &mut trailends))
        .sum::<u32>();

    Some(Box::new(answer))
}

pub(super) fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let map = Map::parse(input);

    let mut trailends = HashMap::<Pos, u32>::new();

    let answer = map
        .iter_trailheads()
        .map(|pos| map.rate_trailhead(pos, &mut trailends))
        .sum::<u32>();

    Some(Box::new(answer))
}

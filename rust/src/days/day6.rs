use std::{
    collections::HashSet,
    fmt,
    ops::{Index, IndexMut},
};

mod viz_gtk;

crate::day_executors! {
    [part1]
    [part2_fast, part2_brute]
}

crate::day_visualizers! {
    []
    [part2_fast_viz]
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cell {
    Obstacle,
    Empty,
    EmptyVisited,
}

impl Cell {
    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        *self == Self::Empty
    }

    #[allow(dead_code)]
    fn is_visited(&self) -> bool {
        *self == Self::EmptyVisited
    }

    #[allow(dead_code)]
    fn is_obstacle(&self) -> bool {
        *self == Self::Obstacle
    }

    fn visit(&mut self) {
        match self {
            Self::Empty => *self = Self::EmptyVisited,
            Self::EmptyVisited => {}
            Self::Obstacle => unreachable!(),
        }
    }
}

#[derive(Clone)]
struct Map {
    height: usize,
    width: usize,
    grid: Vec<Cell>,
    _viz_obstacle: Pos,
    _viz_walk_path: Vec<(Pos, Direction)>,
    _viz_probe_path: Vec<(Pos, Direction)>,
}

impl Map {
    const fn empty() -> Self {
        Self {
            height: 0,
            width: 0,
            grid: Vec::new(),
            _viz_obstacle: Pos::ZERO,
            _viz_walk_path: Vec::new(),
            _viz_probe_path: Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn print(&self, pos: Pos) {
        let suffix = "\x1b[0m";
        for (r, row) in self.grid.chunks(self.width).enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let prefix = if Pos::new(r, c) == pos {
                    "\x1b[100m\x1b[97m"
                } else {
                    ""
                };
                match cell {
                    Cell::Obstacle => print!("#"),
                    Cell::Empty => print!("{prefix}.{suffix}"),
                    Cell::EmptyVisited => print!("{prefix}o{suffix}"),
                }
            }
            println!();
        }
    }

    #[allow(dead_code, clippy::too_many_arguments)]
    fn print_with_probe(
        &self,
        label: Option<&str>,
        (cursor, cursor_style): (Pos, &str),
        (prospective_obstacle, prospective_obstacle_style): (Pos, &str),
        (probe, probe_style): (Pos, &str),
    ) {
        if let Some(l) = label {
            println!(":: {l} ::");
        }
        let suffix = "\x1b[0m";
        for (r, row) in self.grid.chunks(self.width).enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let pos = Pos::new(r, c);
                let prefix = if pos == probe {
                    probe_style
                } else if pos == cursor {
                    cursor_style
                } else if pos == prospective_obstacle {
                    prospective_obstacle_style
                } else {
                    ""
                };
                match cell {
                    Cell::Obstacle => print!("{prefix}#{suffix}"),
                    Cell::Empty => print!("{prefix}.{suffix}"),
                    Cell::EmptyVisited => print!("{prefix}o{suffix}"),
                }
            }
            println!();
        }
    }

    fn contains_pos(&self, pos: Pos) -> bool {
        (0..self.height as isize).contains(&pos.row) && (0..self.width as isize).contains(&pos.col)
    }

    fn walk_from(&mut self, mut pos: Pos) {
        let mut direction = Direction::default();

        loop {
            let next = pos.move_in(direction);
            if !self.contains_pos(next) {
                break;
            }
            if self[next] == Cell::Obstacle {
                direction = direction.rotate();
            } else {
                pos = next;
                self[pos].visit();
            }
        }
    }

    // #region Part 2

    fn walk_and_find_loop_candidates_brute(&self, pos: Pos) -> usize {
        let mut obstacle_candidates = 0;
        let mut loop_path_cache =
            HashSet::<(Pos, Direction)>::with_capacity(self.height * self.width);

        for (r, row) in self.grid.chunks(self.width).enumerate() {
            for (c, cell) in row.iter().enumerate() {
                if cell.is_visited() {
                    continue;
                }

                let next_obstacle_pos = Pos::new(r, c);

                if self.detect_loop(&mut loop_path_cache, pos, next_obstacle_pos) {
                    obstacle_candidates += 1;
                }
            }
        }

        obstacle_candidates
    }

    fn detect_loop(
        &self,
        path_cache: &mut HashSet<(Pos, Direction)>,
        mut pos: Pos,
        next_obstacle: Pos,
    ) -> bool {
        let mut direction = Direction::default();
        path_cache.clear();

        loop {
            if !path_cache.insert((pos, direction)) {
                return true;
            }

            let next = pos.move_in(direction);

            if !self.contains_pos(next) {
                return false;
            }

            if self[next] == Cell::Obstacle || next == next_obstacle {
                direction = direction.rotate();
            } else {
                pos = next;
            }
        }
    }

    // #endregion Part 2

    // #region Part 2 - Fast

    fn walk_and_find_loop_candidates(&mut self, mut pos: Pos) -> usize {
        let mut walk_path = HashSet::from([pos]);
        let mut obstacle_candidates = HashSet::<Pos>::new();
        let mut loop_path_cache = HashSet::<(Pos, Direction)>::new();

        let mut direction = Direction::default();

        loop {
            let mut next_obstacle = pos.move_in(direction);
            if !self.contains_pos(next_obstacle) {
                break;
            }

            if self[next_obstacle].is_obstacle() {
                direction = direction.rotate();
                next_obstacle = pos.move_in(direction);

                if !self.contains_pos(next_obstacle) {
                    break;
                }

                if self[next_obstacle].is_obstacle() {
                    direction = direction.rotate();
                }
            }

            if !walk_path.contains(&next_obstacle) {
                let found_loop =
                    self.probe_loop_fast(&mut loop_path_cache, pos, direction, next_obstacle);
                if found_loop {
                    obstacle_candidates.insert(next_obstacle);
                }
            }

            pos = next_obstacle;
            walk_path.insert(pos);
        }

        obstacle_candidates.len()
    }

    fn probe_loop_fast(
        &mut self,
        loop_path: &mut HashSet<(Pos, Direction)>,
        pos: Pos,
        direction: Direction,
        next_obstacle: Pos,
    ) -> bool {
        let mut probe_dir = direction.rotate();
        let mut probe_pos = pos;

        loop_path.clear();
        loop_path.insert((pos, direction));
        loop_path.insert((probe_pos, probe_dir));

        loop {
            let probe_next = probe_pos.move_in(probe_dir);

            if !self.contains_pos(probe_next) {
                return false;
            }

            if self[probe_next].is_obstacle() || probe_next == next_obstacle {
                probe_dir = probe_dir.rotate();
                loop_path.insert((probe_pos, probe_dir));
                continue;
            } else if loop_path.contains(&(probe_next, probe_dir)) {
                return true;
            }

            probe_pos = probe_next;
            loop_path.insert((probe_pos, probe_dir));
        }
    }

    // #endregion Part 2 - Fast

    // #region Viz

    #[allow(dead_code)]
    fn viz_run_to_obstacle(&mut self, pos: &mut Pos, direction: &mut Direction) {
        loop {
            let next = pos.move_in(*direction);
            if !self.contains_pos(next) {
                break;
            }
            if self[next] == Cell::Obstacle {
                *direction = direction.rotate();
                break;
            } else {
                self[next].visit();
                *pos = next;
            }
        }
    }

    #[allow(dead_code)]
    fn viz_walk_and_find_loop_candidates(
        &mut self,
        path: &mut HashSet<(Pos, Direction)>,
        pos: &mut Pos,
        direction: &mut Direction,
    ) -> bool {
        path.insert((*pos, *direction));
        self._viz_walk_path.push((*pos, *direction));

        let mut loop_path_cache = HashSet::new();

        let mut next_obstacle = pos.move_in(*direction);
        if !self.contains_pos(next_obstacle) {
            return false;
        }

        if self[next_obstacle].is_obstacle() {
            *direction = direction.rotate();
            self._viz_walk_path.push((*pos, *direction));
            next_obstacle = pos.move_in(*direction);

            if !self.contains_pos(next_obstacle) {
                return false;
            }

            if self[next_obstacle].is_obstacle() {
                *direction = direction.rotate();
            }
        }

        self._viz_obstacle = next_obstacle;

        let found_loop =
            self.viz_probe_loop_fast(&mut loop_path_cache, *pos, *direction, next_obstacle);

        *pos = next_obstacle;

        found_loop
    }

    fn viz_probe_loop_fast(
        &mut self,
        loop_path: &mut HashSet<(Pos, Direction)>,
        pos: Pos,
        direction: Direction,
        next_obstacle: Pos,
    ) -> bool {
        let mut probe_dir = direction.rotate();
        let mut probe_pos = pos;

        loop_path.clear();
        loop_path.insert((pos, direction));
        loop_path.insert((probe_pos, probe_dir));

        self._viz_probe_path.clear();

        loop {
            let probe_next = probe_pos.move_in(probe_dir);

            if !self.contains_pos(probe_next) {
                self._viz_probe_path.push((probe_pos, probe_dir));
                return false;
            }

            if self[probe_next].is_obstacle() || probe_next == next_obstacle {
                probe_dir = probe_dir.rotate();
                loop_path.insert((probe_pos, probe_dir));
                self._viz_probe_path.push((probe_pos, probe_dir));
                continue;
            } else if loop_path.contains(&(probe_next, probe_dir)) {
                self._viz_probe_path.push((probe_pos, probe_dir));
                return true;
            }

            probe_pos = probe_next;
            loop_path.insert((probe_pos, probe_dir));
            self._viz_probe_path.push((probe_pos, probe_dir));
        }
    }

    // #endregion Viz
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
struct Pos {
    row: isize,
    col: isize,
}

impl Pos {
    const ZERO: Self = Self { row: 0, col: 0 };
    const INVALID: Self = Self { row: -1, col: -1 };

    const fn new(row: usize, col: usize) -> Self {
        Self {
            row: row as isize,
            col: col as isize,
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

    fn move_in(self, direction: Direction) -> Self {
        match direction {
            Direction::North => self.nn(),
            Direction::East => self.ee(),
            Direction::South => self.ss(),
            Direction::West => self.ww(),
        }
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:3},{:3}", self.row, self.col)
    }
}

impl Index<Pos> for Map {
    type Output = Cell;

    fn index(&self, index: Pos) -> &Self::Output {
        debug_assert!(self.contains_pos(index));
        &self.grid[index.row as usize * self.width + index.col as usize]
    }
}

impl IndexMut<Pos> for Map {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        debug_assert!(self.contains_pos(index));
        &mut self.grid[index.row as usize * self.width + index.col as usize]
    }
}

fn parse(input: &str) -> (Map, Pos) {
    let height = input.lines().count();
    let width = input.lines().next().unwrap().trim().len();

    let mut map = Map {
        height,
        width,
        grid: vec![Cell::Empty; width * height],
        _viz_obstacle: Pos::INVALID,
        _viz_walk_path: Vec::new(),
        _viz_probe_path: Vec::new(),
    };

    let mut start_pos = Pos::default();

    for (r, line) in input.lines().enumerate() {
        for (c, cell) in line.as_bytes().iter().copied().enumerate() {
            let cell_pos = Pos::new(r, c);
            if cell == b'#' {
                map[cell_pos] = Cell::Obstacle;
            } else if cell == b'^' {
                map[cell_pos] = Cell::EmptyVisited;
                start_pos = Pos::new(r, c);
            }
        }
    }

    map._viz_walk_path.push((start_pos, Direction::default()));

    (map, start_pos)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Default for Direction {
    fn default() -> Self {
        Self::North
    }
}

impl Direction {
    fn rotate(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::North => write!(f, "^"),
            Self::East => write!(f, ">"),
            Self::South => write!(f, "v"),
            Self::West => write!(f, "<"),
        }
    }
}

pub(super) fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, start_pos) = parse(input);

    map.walk_from(start_pos);

    let answer = map.grid.iter().filter(|&c| c.is_visited()).count();

    Some(Box::new(answer))
}

pub(super) fn part2_brute(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, start_pos) = parse(input);

    map.walk_from(start_pos);

    let answer = map.walk_and_find_loop_candidates_brute(start_pos);

    Some(Box::new(answer))
}

pub(super) fn part2_fast(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, start_pos) = parse(input);

    let answer = map.walk_and_find_loop_candidates(start_pos);

    Some(Box::new(answer))
}

pub(super) fn part2_fast_viz(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, start_pos) = parse(input);

    viz_gtk::viz_main(&mut map, start_pos, Direction::default());

    None
}

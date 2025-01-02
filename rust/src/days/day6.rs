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
    grid: Vec<Vec<Cell>>, // TODO: flatten this to a 1-D vec
    _viz_obstacle: Cursor,
    _viz_walk_path: Vec<(Cursor, Direction)>,
    _viz_probe_path: Vec<(Cursor, Direction)>,
}

impl Map {
    const fn empty() -> Self {
        Self {
            height: 0,
            width: 0,
            grid: Vec::new(),
            _viz_obstacle: Cursor::zero(),
            _viz_walk_path: Vec::new(),
            _viz_probe_path: Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn print(&self, cursor: Cursor) {
        let suffix = "\x1b[0m";
        for (r, row) in self.grid.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let cell_cursor = Cursor::new(r, c);
                let prefix = if cell_cursor == cursor {
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
        (cursor, cursor_style): (Cursor, &str),
        (prospective_obstacle, prospective_obstacle_style): (Cursor, &str),
        (probe, probe_style): (Cursor, &str),
    ) {
        if let Some(l) = label {
            println!(":: {l} ::");
        }
        let suffix = "\x1b[0m";
        for (r, row) in self.grid.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let cell_cursor = Cursor::new(r, c);
                let prefix = if cell_cursor == probe {
                    probe_style
                } else if cell_cursor == cursor {
                    cursor_style
                } else if cell_cursor == prospective_obstacle {
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

    fn contains_cursor(&self, cursor: Cursor) -> bool {
        (0..self.height as isize).contains(&cursor.row)
            && (0..self.width as isize).contains(&cursor.col)
    }

    fn walk_from(&mut self, mut cursor: Cursor) {
        let mut direction = Direction::default();

        loop {
            let next = cursor.move_in(direction);
            if !self.contains_cursor(next) {
                break;
            }
            if self[next] == Cell::Obstacle {
                direction = direction.rotate();
            } else {
                cursor = next;
                self[cursor].visit();
            }
        }
    }

    // #region Part 2

    fn walk_and_find_loop_candidates_brute(&self, cursor: Cursor) -> usize {
        let mut obstacle_candidates = 0;
        let mut loop_path_cache =
            HashSet::<(Cursor, Direction)>::with_capacity(self.height * self.width);

        for (r, row) in self.grid.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                if cell.is_visited() {
                    continue;
                }

                let next_obstacle_cursor = Cursor::new(r, c);

                if self.detect_loop(&mut loop_path_cache, cursor, next_obstacle_cursor) {
                    obstacle_candidates += 1;
                }
            }
        }

        obstacle_candidates
    }

    fn detect_loop(
        &self,
        path_cache: &mut HashSet<(Cursor, Direction)>,
        mut cursor: Cursor,
        next_obstacle: Cursor,
    ) -> bool {
        let mut direction = Direction::default();
        path_cache.clear();

        loop {
            if !path_cache.insert((cursor, direction)) {
                return true;
            }

            let next = cursor.move_in(direction);

            if !self.contains_cursor(next) {
                return false;
            }

            if self[next] == Cell::Obstacle || next == next_obstacle {
                direction = direction.rotate();
            } else {
                cursor = next;
            }
        }
    }

    // #endregion Part 2

    // #region Part 2 - Fast

    fn walk_and_find_loop_candidates(&mut self, mut cursor: Cursor) -> usize {
        let mut walk_path = HashSet::from([cursor]);
        let mut obstacle_candidates = HashSet::<Cursor>::new();
        let mut loop_path_cache = HashSet::<(Cursor, Direction)>::new();

        let mut direction = Direction::default();

        loop {
            let mut next_obstacle = cursor.move_in(direction);
            if !self.contains_cursor(next_obstacle) {
                break;
            }

            if self[next_obstacle].is_obstacle() {
                direction = direction.rotate();
                next_obstacle = cursor.move_in(direction);

                if !self.contains_cursor(next_obstacle) {
                    break;
                }

                if self[next_obstacle].is_obstacle() {
                    direction = direction.rotate();
                }
            }

            if !walk_path.contains(&next_obstacle) {
                let found_loop =
                    self.probe_loop_fast(&mut loop_path_cache, cursor, direction, next_obstacle);
                if found_loop {
                    obstacle_candidates.insert(next_obstacle);
                }
            }

            cursor = next_obstacle;
            walk_path.insert(cursor);
        }

        obstacle_candidates.len()
    }

    fn probe_loop_fast(
        &mut self,
        loop_path: &mut HashSet<(Cursor, Direction)>,
        cursor: Cursor,
        direction: Direction,
        next_obstacle: Cursor,
    ) -> bool {
        let mut probe_dir = direction.rotate();
        let mut probe_cursor = cursor;

        loop_path.clear();
        loop_path.insert((cursor, direction));
        loop_path.insert((probe_cursor, probe_dir));

        loop {
            let probe_next = probe_cursor.move_in(probe_dir);

            if !self.contains_cursor(probe_next) {
                return false;
            }

            if self[probe_next].is_obstacle() || probe_next == next_obstacle {
                probe_dir = probe_dir.rotate();
                loop_path.insert((probe_cursor, probe_dir));
                continue;
            } else if loop_path.contains(&(probe_next, probe_dir)) {
                return true;
            }

            probe_cursor = probe_next;
            loop_path.insert((probe_cursor, probe_dir));
        }
    }

    // #endregion Part 2 - Fast

    // #region Viz

    #[allow(dead_code)]
    fn viz_run_to_obstacle(&mut self, cursor: &mut Cursor, direction: &mut Direction) {
        loop {
            let next = cursor.move_in(*direction);
            if !self.contains_cursor(next) {
                break;
            }
            if self[next] == Cell::Obstacle {
                *direction = direction.rotate();
                break;
            } else {
                self[next].visit();
                *cursor = next;
            }
        }
    }

    #[allow(dead_code)]
    fn viz_walk_and_find_loop_candidates(
        &mut self,
        path: &mut HashSet<(Cursor, Direction)>,
        cursor: &mut Cursor,
        direction: &mut Direction,
    ) -> bool {
        path.insert((*cursor, *direction));
        self._viz_walk_path.push((*cursor, *direction));

        let mut loop_path_cache = HashSet::new();

        let mut next_obstacle = cursor.move_in(*direction);
        if !self.contains_cursor(next_obstacle) {
            return false;
        }

        if self[next_obstacle].is_obstacle() {
            *direction = direction.rotate();
            self._viz_walk_path.push((*cursor, *direction));
            next_obstacle = cursor.move_in(*direction);

            if !self.contains_cursor(next_obstacle) {
                return false;
            }

            if self[next_obstacle].is_obstacle() {
                *direction = direction.rotate();
            }
        }

        self._viz_obstacle = next_obstacle;

        let found_loop =
            self.viz_probe_loop_fast(&mut loop_path_cache, *cursor, *direction, next_obstacle);

        *cursor = next_obstacle;

        found_loop
    }

    fn viz_probe_loop_fast(
        &mut self,
        loop_path: &mut HashSet<(Cursor, Direction)>,
        cursor: Cursor,
        direction: Direction,
        next_obstacle: Cursor,
    ) -> bool {
        let mut probe_dir = direction.rotate();
        let mut probe_cursor = cursor;

        loop_path.clear();
        loop_path.insert((cursor, direction));
        loop_path.insert((probe_cursor, probe_dir));

        self._viz_probe_path.clear();

        loop {
            let probe_next = probe_cursor.move_in(probe_dir);

            if !self.contains_cursor(probe_next) {
                self._viz_probe_path.push((probe_cursor, probe_dir));
                return false;
            }

            if self[probe_next].is_obstacle() || probe_next == next_obstacle {
                probe_dir = probe_dir.rotate();
                loop_path.insert((probe_cursor, probe_dir));
                self._viz_probe_path.push((probe_cursor, probe_dir));
                continue;
            } else if loop_path.contains(&(probe_next, probe_dir)) {
                self._viz_probe_path.push((probe_cursor, probe_dir));
                return true;
            }

            probe_cursor = probe_next;
            loop_path.insert((probe_cursor, probe_dir));
            self._viz_probe_path.push((probe_cursor, probe_dir));
        }
    }

    // #endregion Viz
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
struct Cursor {
    row: isize,
    col: isize,
}

impl Cursor {
    const fn zero() -> Self {
        Self::new(0, 0)
    }

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

impl fmt::Display for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:3},{:3}", self.row, self.col)
    }
}

impl Index<Cursor> for Map {
    type Output = Cell;

    fn index(&self, index: Cursor) -> &Self::Output {
        debug_assert!(self.contains_cursor(index));
        &self.grid[index.row as usize][index.col as usize]
    }
}

impl IndexMut<Cursor> for Map {
    fn index_mut(&mut self, index: Cursor) -> &mut Self::Output {
        debug_assert!(self.contains_cursor(index));
        &mut self.grid[index.row as usize][index.col as usize]
    }
}

fn parse(input: &str) -> (Map, Cursor) {
    let height = input.lines().count();
    let width = input.lines().next().unwrap().trim().len();

    let mut grid = vec![vec![Cell::Empty; width]; height];

    let mut cursor = Cursor::default();

    for (r, line) in input.lines().enumerate() {
        for (c, cell) in line.as_bytes().iter().copied().enumerate() {
            if cell == b'#' {
                grid[r][c] = Cell::Obstacle;
            } else if cell == b'^' {
                grid[r][c] = Cell::EmptyVisited;
                cursor = Cursor::new(r, c);
            }
        }
    }

    (
        Map {
            height,
            width,
            grid,
            _viz_obstacle: Cursor { row: -1, col: -1 },
            _viz_walk_path: vec![(cursor, Direction::default())],
            _viz_probe_path: Vec::new(),
        },
        cursor,
    )
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
    let (mut map, cursor) = parse(input);

    map.walk_from(cursor);

    let answer = map
        .grid
        .iter()
        .map(|row| row.iter().filter(|c| c.is_visited()).count())
        .sum::<usize>();

    Some(Box::new(answer))
}

pub(super) fn part2_brute(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, cursor) = parse(input);

    map.walk_from(cursor);

    let answer = map.walk_and_find_loop_candidates_brute(cursor);

    Some(Box::new(answer))
}

pub(super) fn part2_fast(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, cursor) = parse(input);

    let answer = map.walk_and_find_loop_candidates(cursor);

    Some(Box::new(answer))
}

pub(super) fn part2_fast_viz(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, cursor) = parse(input);

    viz_gtk::viz_main(&mut map, cursor, Direction::default());

    None
}

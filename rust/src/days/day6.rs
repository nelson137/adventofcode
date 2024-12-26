use std::{
    collections::HashSet,
    fmt, mem,
    ops::{Index, IndexMut},
};

crate::day_executors! {
    [part1]
    [part2, part2_fast]
}

#[derive(Clone, Debug)]
enum Cell {
    Obstacle,
    Empty,
    EmptyVisited(CellVisits),
}

impl Cell {
    fn is_visited(&self) -> bool {
        matches!(*self, Self::EmptyVisited(_))
    }

    fn is_obstacle(&self) -> bool {
        matches!(*self, Self::Obstacle)
    }

    fn push_visit(&mut self, d: Direction, s: u64) {
        match self {
            Self::Empty => *self = Self::EmptyVisited(CellVisits::from_visit(d, s)),
            Self::EmptyVisited(visits) => visits.0.push((d, s)),
            Self::Obstacle => unreachable!(),
        }
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::EmptyVisited(_), _) | (_, Self::EmptyVisited(_)) => false,
            _ => mem::discriminant(self) == mem::discriminant(other),
        }
    }
}

#[derive(Clone, Debug)]
struct CellVisits(Vec<(Direction, u64)>);

impl CellVisits {
    fn from_visit(direction: Direction, step: u64) -> Self {
        Self(vec![(direction, step)])
    }
}

impl fmt::Display for CellVisits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut flags = 0;
        for visit in &self.0 {
            flags |= match visit.0 {
                Direction::North => 0x1, // 0b0001
                Direction::East => 0x2,  // 0b0010
                Direction::South => 0x4, // 0b0100
                Direction::West => 0x8,  // 0b1000
            };
        }
        match flags {
            0b0001 => write!(f, "↑"), // N...
            0b0010 => write!(f, "→"), // .E..
            0b0011 => write!(f, "↗"), // NE..
            0b0100 => write!(f, "↓"), // ..S.
            0b0101 => write!(f, "↕"), // N.S.
            0b0110 => write!(f, "↘"), // .ES.
            0b0111 => write!(f, "?"), // NES.
            0b1000 => write!(f, "←"), // ...W
            0b1001 => write!(f, "↖"), // N..W
            0b1010 => write!(f, "↔"), // .E.W
            0b1011 => write!(f, "?"), // NE.W
            0b1100 => write!(f, "↙"), // ..SW
            0b1101 => write!(f, "?"), // N.SW
            0b1110 => write!(f, "?"), // .ESW
            0b1111 => write!(f, "?"), // NESW
            _ => unreachable!(),
        }
    }
}

struct Map {
    height: usize,
    width: usize,
    grid: Vec<Vec<Cell>>,
}

impl Map {
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
                    Cell::EmptyVisited(visits) => print!("{prefix}{visits}{suffix}"),
                }
            }
            println!();
        }
    }

    #[allow(dead_code, clippy::too_many_arguments)]
    fn print_with_probe(
        &self,
        label: Option<&str>,
        cursor: Cursor,
        cursor_style: &str,
        prospective_obstacle: Cursor,
        prospective_obstacle_style: &str,
        probe: Cursor,
        probe_style: &str,
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
                    Cell::EmptyVisited(visits) => print!("{prefix}{visits}{suffix}"),
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
        let mut step = 1; // skip current position

        loop {
            let next = cursor.move_in(direction);
            if !self.contains_cursor(next) {
                break;
            }
            if self[next] == Cell::Obstacle {
                direction = direction.rotate();
            } else {
                cursor = next;
                self[cursor].push_visit(direction, step);
                step += 1;
            }
        }
    }

    fn walk_from_and_find_loop_candidates(&mut self, mut cursor: Cursor) -> usize {
        let mut direction = Direction::default();

        // TODO: is a hash set actually necessary or can we just count?
        let mut obstacle_candidates = HashSet::<Cursor>::new();

        loop {
            let next_obstacle = cursor.move_in(direction);
            if !self.contains_cursor(next_obstacle) {
                break;
            }
            if self[next_obstacle] == Cell::Obstacle {
                direction = direction.rotate();
                continue;
            }

            let original_cell = self[next_obstacle].clone();
            self[next_obstacle] = Cell::Obstacle;

            if self.probe_loop(cursor, direction, next_obstacle) {
                obstacle_candidates.insert(next_obstacle);
            }

            self[next_obstacle] = original_cell.clone();

            cursor = next_obstacle;
        }

        // for o in &obstacle_candidates {
        //     println!("{o}");
        // }

        obstacle_candidates.len()
    }

    fn probe_loop(
        &self,
        cursor: Cursor,
        direction: Direction,
        prospective_obstacle: Cursor,
    ) -> bool {
        let mut probe_cursor = cursor;
        let mut probe_dir = direction.rotate();

        let mut loop_path = HashSet::<(Cursor, Direction)>::new();

        loop {
            #[allow(dead_code)]
            mod cell_style {
                //! [ANSI colors](https://en.wikipedia.org/wiki/ANSI_escape_code#Colors)
                pub const RED: &str = "\x1b[41m\x1b[30m";
                pub const GREEN: &str = "\x1b[42m\x1b[30m";
                pub const YELLOW: &str = "\x1b[43m\x1b[30m";
                pub const CYAN: &str = "\x1b[46m\x1b[30m";
                pub const BRIGHT_BLACK: &str = "\x1b[100m\x1b[97m";
                pub const WHITE: &str = "\x1b[107m\x1b[30m";
                pub const X_CURSOR: &str = WHITE;
            }

            #[allow(unused_variables, unreachable_code)]
            let print_probe = |label, p_obstacle_style, probe_style| {
                return;
                self.print_with_probe(
                    label,
                    cursor,
                    cell_style::X_CURSOR,
                    prospective_obstacle,
                    p_obstacle_style,
                    probe_cursor,
                    probe_style,
                );
                std::io::stdin().read_line(&mut String::new()).unwrap();
            };

            print_probe(None, cell_style::YELLOW, cell_style::BRIGHT_BLACK);

            loop_path.insert((probe_cursor, probe_dir));

            let probe_next = probe_cursor.move_in(probe_dir);

            if !self.contains_cursor(probe_next) {
                print_probe(Some("Out of Bounds"), cell_style::RED, cell_style::RED);
                return false;
            }

            match &self[probe_next] {
                Cell::Obstacle => {
                    probe_dir = probe_dir.rotate();
                    if loop_path.contains(&(probe_cursor, probe_dir)) {
                        print_probe(
                            Some("Found Obstacle Candidate"),
                            cell_style::GREEN,
                            cell_style::GREEN,
                        );
                        return true;
                    }
                    continue;
                }

                Cell::Empty | Cell::EmptyVisited(_) => {}
            }

            probe_cursor = probe_next;
        }
    }

    fn walk_from_and_find_loop_candidates_brute(&self, cursor: Cursor) -> usize {
        // TODO: is a hash set actually necessary or can we just count?
        let mut obstacle_candidates = HashSet::<Cursor>::new();

        for (r, row) in self.grid.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let cell_cursor = Cursor::new(r, c);
                if !cell.is_obstacle() && self.detect_loop(cursor, cell_cursor) {
                    obstacle_candidates.insert(cell_cursor);
                }
            }
        }

        // for o in &obstacle_candidates {
        //     println!("{o}");
        // }

        obstacle_candidates.len()
    }

    fn detect_loop(&self, mut cursor: Cursor, prospective_obstacle: Cursor) -> bool {
        let mut direction = Direction::default();
        let mut path = HashSet::<(Cursor, Direction)>::new();

        loop {
            #[allow(dead_code)]
            mod cell_style {
                //! [ANSI colors](https://en.wikipedia.org/wiki/ANSI_escape_code#Colors)
                pub const RED: &str = "\x1b[41m\x1b[30m";
                pub const GREEN: &str = "\x1b[42m\x1b[30m";
                pub const YELLOW: &str = "\x1b[43m\x1b[30m";
                pub const CYAN: &str = "\x1b[46m\x1b[30m";
                pub const BRIGHT_BLACK: &str = "\x1b[100m\x1b[97m";
                pub const WHITE: &str = "\x1b[107m\x1b[30m";
                pub const X_CURSOR: &str = WHITE;
            }

            #[allow(unused_variables, unreachable_code)]
            let print_probe = |label, p_obstacle_style, probe_style| {
                return;
                self.print_with_probe(
                    label,
                    cursor,
                    cell_style::X_CURSOR,
                    prospective_obstacle,
                    p_obstacle_style,
                    cursor,
                    probe_style,
                );
                std::io::stdin().read_line(&mut String::new()).unwrap();
            };

            print_probe(None, cell_style::YELLOW, cell_style::BRIGHT_BLACK);

            // TODO: use `HashSet` entry API ([tracking issue](https://github.com/rust-lang/rust/issues/60896))
            if path.contains(&(cursor, direction)) {
                print_probe(
                    Some("Found Obstacle Candidate"),
                    cell_style::GREEN,
                    cell_style::GREEN,
                );
                return true;
            }
            path.insert((cursor, direction));
            let next = cursor.move_in(direction);
            if !self.contains_cursor(next) {
                print_probe(Some("Out of Bounds"), cell_style::RED, cell_style::RED);
                return false;
            }
            if self[next] == Cell::Obstacle || next == prospective_obstacle {
                direction = direction.rotate();
            } else {
                cursor = next;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
struct Cursor {
    row: isize,
    col: isize,
}

impl Cursor {
    fn new(row: usize, col: usize) -> Self {
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
        write!(f, "{},{}", self.row, self.col)
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
                grid[r][c] = Cell::EmptyVisited(CellVisits::from_visit(Direction::default(), 0));
                cursor = Cursor::new(r, c);
            }
        }
    }

    (
        Map {
            height,
            width,
            grid,
        },
        cursor,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Direction {
    North = 0x1, // 0b0001
    East = 0x2,  // 0b0010
    South = 0x4, // 0b0100
    West = 0x8,  // 0b1000
}

impl Default for Direction {
    fn default() -> Self {
        Self::North
    }
}

impl Direction {
    // fn is_north(self) -> bool {
    //     self.intersects(Self::North)
    // }
    //
    // fn is_east(self) -> bool {
    //     self.intersects(Self::East)
    // }
    //
    // fn is_south(self) -> bool {
    //     self.intersects(Self::South)
    // }
    //
    // fn is_west(self) -> bool {
    //     self.intersects(Self::West)
    // }
    //
    // fn is_opposite_of(self, other: Self) -> bool {
    //     matches!(
    //         (self, other),
    //         (Self::North, Self::South)
    //             | (Self::South, Self::North)
    //             | (Self::East, Self::West)
    //             | (Self::West, Self::East)
    //     )
    // }

    fn rotate(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
}

// impl fmt::Display for Direction {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             Self::North => write!(f, "^"),
//             Self::East => write!(f, ">"),
//             Self::South => write!(f, "v"),
//             Self::West => write!(f, "<"),
//         }
//     }
// }

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

pub(super) fn part2_fast(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, cursor) = parse(input);

    let answer = map.walk_from_and_find_loop_candidates(cursor);

    Some(Box::new(answer))
}

pub(super) fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (map, cursor) = parse(input);

    let answer = map.walk_from_and_find_loop_candidates_brute(cursor);

    Some(Box::new(answer))
}

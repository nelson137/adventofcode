use std::{
    collections::HashSet,
    fmt,
    ops::{Index, IndexMut},
};

crate::day_executors! {
    [part1]
    [part2]
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cell {
    Obstruction,
    Empty,
    EmptyVisited { direction: Direction, step: u64 },
}

impl Cell {
    fn is_visited(&self) -> bool {
        matches!(*self, Self::EmptyVisited { .. })
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
            for (c, cell) in row.iter().copied().enumerate() {
                let cell_cursor = Cursor::new(r, c);
                let prefix = if cell_cursor == cursor {
                    "\x1b[100m\x1b[97m"
                } else {
                    ""
                };
                match cell {
                    Cell::Obstruction => print!("#"),
                    Cell::Empty => print!("{prefix}.{suffix}"),
                    Cell::EmptyVisited { .. } => print!("{prefix}o{suffix}"),
                }
            }
            println!();
        }
    }

    #[allow(dead_code)]
    fn print_with_probe(&self, cursor: Cursor, probe: Cursor, probe_color_prefix: &str) {
        let suffix = "\x1b[0m";
        for (r, row) in self.grid.iter().enumerate() {
            for (c, cell) in row.iter().copied().enumerate() {
                let cell_cursor = Cursor::new(r, c);
                let prefix = if cell_cursor == cursor {
                    "\x1b[100m\x1b[97m"
                } else if cell_cursor == probe {
                    probe_color_prefix
                } else {
                    ""
                };
                match cell {
                    Cell::Obstruction => print!("{prefix}#{suffix}"),
                    Cell::Empty => print!("{prefix}.{suffix}"),
                    Cell::EmptyVisited { direction, .. } => print!("{prefix}{direction}{suffix}"),
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
        let mut direction = Direction::North;
        let mut step = 1; // skip current position

        loop {
            let next = cursor.move_in(direction);
            if !self.contains_cursor(next) {
                break;
            }
            if self[next] == Cell::Obstruction {
                direction = direction.rotate();
            } else {
                cursor = next;
                self[cursor] = Cell::EmptyVisited { direction, step };
                step += 1;
            }
        }
    }

    fn walk_from_and_find_loop_candidates(&self, mut cursor: Cursor) -> usize {
        let mut direction = Direction::North;
        let mut step = 1; // skip current position

        let mut obstacle_candidates = HashSet::<Cursor>::new();

        loop {
            let next = cursor.move_in(direction);
            if !self.contains_cursor(next) {
                break;
            }
            if self[next] == Cell::Obstruction {
                direction = direction.rotate();
                continue;
            }

            let probe_dir = direction.rotate();
            let mut probe_next = cursor;

            loop {
                probe_next = probe_next.move_in(probe_dir);
                if !self.contains_cursor(probe_next) {
                    break;
                }

                // // https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
                // self.print_with_probe(cursor, probe_next, "\x1b[43m\x1b[30m");
                // std::io::stdin().read_line(&mut String::new()).unwrap();

                // Next probe cell...
                match self[probe_next] {
                    // ...is visited going in the opposite direction
                    // => can't be a loop, break
                    Cell::EmptyVisited { direction: d, .. } if d.is_opposite_of(probe_dir) => {
                        // self.print_with_probe(cursor, probe_next, "\x1b[41m\x1b[30m");
                        // std::io::stdin().read_line(&mut String::new()).unwrap();
                        break;
                    }

                    // ...is visited, going in the probe direction, and has a
                    //    step index less than the current
                    // => loop found, obstacle candidate found
                    Cell::EmptyVisited {
                        direction: d,
                        step: s,
                    } if d == probe_dir => {
                        if s < step {
                            obstacle_candidates.insert(next);
                            // self.print_with_probe(cursor, probe_next, "\x1b[42m\x1b[30m");
                            // std::io::stdin().read_line(&mut String::new()).unwrap();
                        } else {
                            // self.print_with_probe(cursor, probe_next, "\x1b[41m\x1b[30m");
                            // std::io::stdin().read_line(&mut String::new()).unwrap();
                        }
                        break;
                    }

                    // ...is an obstruction
                    // => can't be a loop, break
                    Cell::Obstruction => {
                        // self.print_with_probe(cursor, probe_next, "\x1b[41m\x1b[30m");
                        // std::io::stdin().read_line(&mut String::new()).unwrap();
                        break;
                    }

                    // ...doesn't impede a potential loop
                    // => keep going
                    Cell::Empty | Cell::EmptyVisited { .. } => {}
                }
            }

            cursor = next;
            step += 1;
        }

        // for o in &obstacle_candidates {
        //     println!("{o}");
        // }

        obstacle_candidates.len()
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
        if direction.intersects(Direction::North) {
            self.nn()
        } else if direction.intersects(Direction::East) {
            self.ee()
        } else if direction.intersects(Direction::South) {
            self.ss()
        } else if direction.intersects(Direction::West) {
            self.ww()
        } else {
            unimplemented!()
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
                grid[r][c] = Cell::Obstruction;
            } else if cell == b'^' {
                grid[r][c] = Cell::EmptyVisited {
                    direction: Direction::North,
                    step: 0,
                };
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

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    struct Direction: u8 {
        const North = 0x1; // 0b0001
        const East = 0x2;  // 0b0010
        const South = 0x4; // 0b0100
        const West = 0x8;  // 0b1000
    }
}

// impl Default for Direction {
//     fn default() -> Self {
//         Self::North
//     }
// }

impl Direction {
    fn is_north(self) -> bool {
        self.intersects(Self::North)
    }

    fn is_east(self) -> bool {
        self.intersects(Self::East)
    }

    fn is_south(self) -> bool {
        self.intersects(Self::South)
    }

    fn is_west(self) -> bool {
        self.intersects(Self::West)
    }

    fn is_opposite_of(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::North, Self::South)
                | (Self::South, Self::North)
                | (Self::East, Self::West)
                | (Self::West, Self::East)
        )
    }

    fn rotate(self) -> Self {
        if self.is_north() {
            Self::East
        } else if self.is_east() {
            Self::South
        } else if self.is_south() {
            Self::West
        } else if self.is_west() {
            Self::North
        } else {
            unimplemented!()
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.intersects(Self::North) {
            write!(f, "^")
        } else if self.intersects(Self::East) {
            write!(f, ">")
        } else if self.intersects(Self::South) {
            write!(f, "v")
        } else if self.intersects(Self::West) {
            write!(f, "<")
        } else {
            unimplemented!()
        }
    }
}

pub(super) fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, cursor) = parse(input);

    map.walk_from(cursor);

    let answer = map
        .grid
        .iter()
        .map(|row| row.iter().copied().filter(Cell::is_visited).count())
        .sum::<usize>();

    Some(Box::new(answer))
}

pub(super) fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, cursor) = parse(input);

    map.walk_from(cursor);

    let answer = map.walk_from_and_find_loop_candidates(cursor);

    Some(Box::new(answer))
}

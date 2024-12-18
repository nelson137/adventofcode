use std::{
    fmt,
    ops::{Index, IndexMut},
};

crate::day_executors! {
    [part1, part1_with_inverted]
    [part2]
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cell {
    Obstruction,
    Empty,
    EmptyVisited,
}

struct Map {
    height: usize,
    width: usize,
    grid: Vec<Vec<Cell>>,
    grid_inverted: Vec<Vec<Cell>>,
}

impl Map {
    #[allow(dead_code)]
    fn print(&self, cursor: Cursor) {
        for (r, row) in self.grid.iter().enumerate() {
            for (c, cell) in row.iter().copied().enumerate() {
                let cell_cursor = Cursor::new(r, c);
                match cell {
                    Cell::Obstruction => print!("#"),
                    Cell::Empty if cell_cursor == cursor => {
                        print!("\x1b[103m\x1b[30m.\x1b[0m")
                    }
                    Cell::EmptyVisited if cell_cursor == cursor => {
                        print!("\x1b[103m\x1b[30mo\x1b[0m")
                    }
                    Cell::Empty => print!("."),
                    Cell::EmptyVisited => print!("o"),
                }
            }
            println!();
        }
    }

    fn contains_cursor(&self, cursor: Cursor) -> bool {
        (0..self.height as isize).contains(&cursor.row)
            && (0..self.width as isize).contains(&cursor.col)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
    let mut grid_inverted = vec![vec![Cell::Empty; width]; height];

    let mut cursor = Cursor::default();

    for (r, line) in input.lines().enumerate() {
        for (c, cell) in line.as_bytes().iter().copied().enumerate() {
            if cell == b'#' {
                grid[r][c] = Cell::Obstruction;
                grid_inverted[c][r] = Cell::Obstruction;
            } else if cell == b'^' {
                grid[r][c] = Cell::EmptyVisited;
                grid_inverted[c][r] = Cell::EmptyVisited;
                cursor = Cursor::new(r, c);
            }
        }
    }

    (
        Map {
            height,
            width,
            grid,
            grid_inverted,
        },
        cursor,
    )
}

#[derive(Clone, Copy, Default)]
enum Direction {
    #[default]
    North,
    East,
    South,
    West,
}

impl Direction {
    fn is_ns(self) -> bool {
        matches!(self, Self::North | Self::South)
    }

    fn rotate(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
}

pub(super) fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, mut cursor) = parse(input);
    let mut direction = Direction::default();

    loop {
        let next = cursor.move_in(direction);
        if !map.contains_cursor(next) {
            break;
        }
        if map[next] == Cell::Obstruction {
            direction = direction.rotate();
        } else {
            cursor = next;
            map[cursor] = Cell::EmptyVisited;
        }
    }

    let answer = map
        .grid
        .iter()
        .map(|row| {
            row.iter()
                .copied()
                .filter(|cell| *cell == Cell::EmptyVisited)
                .count()
        })
        .sum::<usize>();

    Some(Box::new(answer))
}

pub(super) fn part1_with_inverted(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let (mut map, mut cursor) = parse(input);
    let mut direction = Direction::default();

    loop {
        let next = cursor.move_in(direction);
        if !map.contains_cursor(next) {
            break;
        }
        #[allow(clippy::collapsible_else_if)]
        if direction.is_ns() {
            if map.grid_inverted[next.col as usize][next.row as usize] == Cell::Obstruction {
                direction = direction.rotate();
            } else {
                cursor = next;
                map.grid[next.row as usize][next.col as usize] = Cell::EmptyVisited;
                map.grid_inverted[next.col as usize][next.row as usize] = Cell::EmptyVisited;
            }
        } else {
            if map.grid[next.row as usize][next.col as usize] == Cell::Obstruction {
                direction = direction.rotate();
            } else {
                cursor = next;
                map.grid[next.row as usize][next.col as usize] = Cell::EmptyVisited;
                map.grid_inverted[next.col as usize][next.row as usize] = Cell::EmptyVisited;
            }
        }
    }

    let answer = map
        .grid
        .iter()
        .map(|row| {
            row.iter()
                .copied()
                .filter(|cell| *cell == Cell::EmptyVisited)
                .count()
        })
        .sum::<usize>();

    Some(Box::new(answer))
}

pub(super) fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    _ = input;

    None
}

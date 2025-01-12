use std::{fmt, ops};

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
    plots: Vec<&'input [u8]>,
    mapped_plots: FlatMap<bool>,
}

impl<'input> Map<'input> {
    fn parse(input: &'input str) -> Self {
        let height = input.lines().count();
        let width = input.lines().next().unwrap().trim().len();

        let mut plots = Vec::with_capacity(height);
        plots.extend(input.lines().map(|line| line.as_bytes()));

        let mapped_plots = FlatMap::new(height, width, false);

        Self {
            height,
            width,
            plots,
            mapped_plots,
        }
    }

    #[inline(always)]
    fn plot(&self, pos: Pos) -> u8 {
        self.plots[pos.row as usize][pos.col as usize]
    }

    #[inline(always)]
    fn pos_on_north_edge(&self, pos: Pos) -> bool {
        pos.row == 0
    }

    #[inline(always)]
    fn pos_on_east_edge(&self, pos: Pos) -> bool {
        pos.col as usize == self.width - 1
    }

    #[inline(always)]
    fn pos_on_south_edge(&self, pos: Pos) -> bool {
        pos.row as usize == self.height - 1
    }

    #[inline(always)]
    fn pos_on_west_edge(&self, pos: Pos) -> bool {
        pos.col == 0
    }

    #[inline(always)]
    fn pos_on_orthogonal1_edge(&self, pos: Pos, direction: Direction) -> bool {
        match direction {
            Direction::North => self.pos_on_east_edge(pos),
            Direction::East => self.pos_on_south_edge(pos),
            Direction::South => self.pos_on_west_edge(pos),
            Direction::West => self.pos_on_north_edge(pos),
        }
    }

    #[inline(always)]
    fn pos_on_orthogonal2_edge(&self, pos: Pos, direction: Direction) -> bool {
        match direction {
            Direction::North => self.pos_on_west_edge(pos),
            Direction::East => self.pos_on_north_edge(pos),
            Direction::South => self.pos_on_east_edge(pos),
            Direction::West => self.pos_on_south_edge(pos),
        }
    }

    fn floodfill_region_cost(&mut self, seed: Pos, to_search: &mut Vec<Pos>) -> u64 {
        if self.mapped_plots[seed] {
            return 0;
        }

        let plot = self.plot(seed);
        let mut area = 0_u64;
        let mut perimeter = 0_u64;

        to_search.clear();
        to_search.push(seed);

        // From [Span Filling on Wikipedia](https://en.wikipedia.org/wiki/Flood_fill#Span_filling):
        //
        // ```
        // fn fill(x, y):
        //     if not Inside(x, y) then return
        //     let s = new empty stack or queue
        //     Add (x, y) to s
        //     while s is not empty:
        //         Remove an (x, y) from s
        //         let lx = x
        //         while Inside(lx - 1, y):
        //             Set(lx - 1, y)
        //             lx = lx - 1
        //         while Inside(x, y):
        //             Set(x, y)
        //             x = x + 1
        //       scan(lx, x - 1, y + 1, s)
        //       scan(lx, x - 1, y - 1, s)
        //
        // fn scan(lx, rx, y, s):
        //     let span_added = false
        //     for x in lx .. rx:
        //         if not Inside(x, y):
        //             span_added = false
        //         else if not span_added:
        //             Add (x, y) to s
        //             span_added = true
        // ```

        while let Some(pos) = to_search.pop() {
            // Set
            {
                if self.mapped_plots[pos] {
                    continue;
                }
                self.mapped_plots[pos] = true;
                area += 1;
            }

            let mut next = pos;
            let l_col = loop {
                if self.pos_on_west_edge(next) {
                    perimeter += 1;
                    break next.col;
                }

                next = next.ww();

                if self.plot(next) == plot {
                    if self.mapped_plots[next] {
                        break next.col + 1;
                    } else {
                        // Set
                        area += 1;
                        self.mapped_plots[next] = true;
                    }
                } else {
                    perimeter += 1;
                    break next.col + 1;
                }
            };

            let mut next = pos;
            let r_col = loop {
                if self.pos_on_east_edge(next) {
                    perimeter += 1;
                    break next.col;
                }

                next = next.ee();

                if self.plot(next) == plot {
                    if self.mapped_plots[next] {
                        break next.col - 1;
                    } else {
                        // Set
                        area += 1;
                        self.mapped_plots[next] = true;
                    }
                } else {
                    perimeter += 1;
                    break next.col - 1;
                }
            };

            if self.pos_on_south_edge(pos) {
                perimeter += (1 + r_col - l_col) as u64;
            } else {
                perimeter += self.floodfill_scan(plot, pos.row + 1, l_col, r_col, to_search);
            }

            if self.pos_on_north_edge(pos) {
                perimeter += (1 + r_col - l_col) as u64;
            } else {
                perimeter += self.floodfill_scan(plot, pos.row - 1, l_col, r_col, to_search);
            }
        }

        area * perimeter
    }

    #[inline(always)]
    fn floodfill_scan(
        &self,
        plot: u8,
        row: u32,
        l_col: u32,
        r_col: u32,
        to_search: &mut Vec<Pos>,
    ) -> u64 {
        let mut next = Pos { row, col: l_col };
        let mut span_added = false;
        let mut added_perimeter = 0;

        while next.col <= r_col {
            if self.plot(next) != plot {
                span_added = false;
                added_perimeter += 1;
            } else if !span_added && !self.mapped_plots[next] {
                to_search.push(next);
                span_added = true;
            }

            next.col += 1;
        }

        added_perimeter
    }

    fn floodfill_solve(&mut self) -> u64 {
        let mut total_fence_cost = 0_u64;

        let mut to_search = Vec::<Pos>::with_capacity(self.mapped_plots.map.len() / 4);

        for r in 0..self.height {
            for c in 0..self.width {
                let seed = Pos::new(r, c);
                total_fence_cost += self.floodfill_region_cost(seed, &mut to_search);
            }
        }

        total_fence_cost
    }

    fn floodfill_region_bulk_cost(&mut self, seed: Pos, to_search: &mut Vec<Pos>) -> u64 {
        if self.mapped_plots[seed] {
            return 0;
        }

        let plot = self.plot(seed);
        let mut area = 0_u64;
        let mut sides = 0_u64;

        to_search.clear();
        to_search.push(seed);

        while let Some(pos) = to_search.pop() {
            // Set
            if self.mapped_plots[pos] {
                continue;
            }
            self.mapped_plots[pos] = true;
            area += 1;

            // West

            if self.pos_on_west_edge(pos) {
                self.floodfill_bulk_orthogonal_scan_map_edge(
                    (plot, pos, Direction::West),
                    &mut area,
                    &mut sides,
                    to_search,
                );
            } else {
                let next = pos.ww();
                if self.plot(next) == plot {
                    if !self.mapped_plots[next] {
                        to_search.push(next);
                    }
                } else {
                    self.floodfill_bulk_orthogonal_scan_plot_edge(
                        (plot, pos, Direction::West),
                        &mut area,
                        &mut sides,
                        to_search,
                    );
                }
            }

            // East

            if self.pos_on_east_edge(pos) {
                self.floodfill_bulk_orthogonal_scan_map_edge(
                    (plot, pos, Direction::East),
                    &mut area,
                    &mut sides,
                    to_search,
                );
            } else {
                let next = pos.ee();
                if self.plot(next) == plot {
                    if !self.mapped_plots[next] {
                        to_search.push(next);
                    }
                } else {
                    self.floodfill_bulk_orthogonal_scan_plot_edge(
                        (plot, pos, Direction::East),
                        &mut area,
                        &mut sides,
                        to_search,
                    );
                }
            }

            // North

            if self.pos_on_north_edge(pos) {
                self.floodfill_bulk_orthogonal_scan_map_edge(
                    (plot, pos, Direction::North),
                    &mut area,
                    &mut sides,
                    to_search,
                );
            } else {
                let next = pos.nn();
                if self.plot(next) == plot {
                    if !self.mapped_plots[next] {
                        to_search.push(next);
                    }
                } else {
                    self.floodfill_bulk_orthogonal_scan_plot_edge(
                        (plot, pos, Direction::North),
                        &mut area,
                        &mut sides,
                        to_search,
                    );
                }
            }

            // South

            if self.pos_on_south_edge(pos) {
                self.floodfill_bulk_orthogonal_scan_map_edge(
                    (plot, pos, Direction::South),
                    &mut area,
                    &mut sides,
                    to_search,
                );
            } else {
                let next = pos.ss();
                if self.plot(next) == plot {
                    if !self.mapped_plots[next] {
                        to_search.push(next);
                    }
                } else {
                    self.floodfill_bulk_orthogonal_scan_plot_edge(
                        (plot, pos, Direction::South),
                        &mut area,
                        &mut sides,
                        to_search,
                    );
                }
            }
        }

        println!(
            "[{}] area={area} sides={sides} | {}",
            plot as char,
            area * sides
        );

        area * sides
    }

    fn floodfill_bulk_orthogonal_scan_map_edge(
        &mut self,
        (plot, pos, direction): (u8, Pos, Direction),
        area: &mut u64,
        sides: &mut u64,
        to_search: &mut Vec<Pos>,
    ) {
        *sides += 1;
        let mut probe;

        probe = pos;

        loop {
            if self.pos_on_orthogonal1_edge(probe, direction) {
                break;
            }

            probe = probe.move_in_orthogonal1(direction);

            if self.mapped_plots[probe] || self.plot(probe) != plot {
                break;
            }

            self.mapped_plots[probe] = true;
            *area += 1;

            let behind = probe.move_opposite_of(direction);
            if self.plot(behind) == plot {
                to_search.push(behind);
            }
        }

        probe = pos;

        loop {
            if self.pos_on_orthogonal2_edge(probe, direction) {
                break;
            }

            probe = probe.move_in_orthogonal2(direction);

            if self.mapped_plots[probe] || self.plot(probe) != plot {
                break;
            }

            self.mapped_plots[probe] = true;
            *area += 1;

            let behind = probe.move_opposite_of(direction);
            if self.plot(behind) == plot {
                to_search.push(behind);
            }
        }
    }

    fn floodfill_bulk_orthogonal_scan_plot_edge(
        &mut self,
        (plot, pos, direction): (u8, Pos, Direction),
        area: &mut u64,
        sides: &mut u64,
        to_search: &mut Vec<Pos>,
    ) {
        *sides += 1;
        let mut probe;

        probe = pos;

        loop {
            if self.pos_on_orthogonal1_edge(probe, direction) {
                break;
            }

            probe = probe.move_in_orthogonal1(direction);

            if self.mapped_plots[probe] || self.plot(probe) != plot {
                break;
            }

            self.mapped_plots[probe] = true;
            *area += 1;

            let ahead = probe.move_in(direction);
            if self.plot(ahead) == plot {
                break;
            }

            let behind = probe.move_opposite_of(direction);
            if self.plot(behind) == plot {
                to_search.push(behind);
            }
        }

        probe = pos;

        loop {
            if self.pos_on_orthogonal2_edge(probe, direction) {
                break;
            }

            probe = probe.move_in_orthogonal2(direction);

            if self.mapped_plots[probe] || self.plot(probe) != plot {
                break;
            }

            self.mapped_plots[probe] = true;
            *area += 1;

            let ahead = probe.move_in(direction);
            if self.plot(ahead) == plot {
                break;
            }

            let behind = probe.move_opposite_of(direction);
            if self.plot(behind) == plot {
                to_search.push(behind);
            }
        }
    }

    fn floodfill_solve_bulk(&mut self) -> u64 {
        let mut total_fence_cost = 0_u64;

        let mut to_search = Vec::<Pos>::with_capacity(self.mapped_plots.map.len() / 4);

        for r in 0..self.height {
            for c in 0..self.width {
                let seed = Pos::new(r, c);
                total_fence_cost += self.floodfill_region_bulk_cost(seed, &mut to_search);
            }
        }

        total_fence_cost
    }
}

struct FlatMap<T> {
    width: usize,
    map: Vec<T>,
}

impl<T: Clone + Copy + PartialEq + Eq> FlatMap<T> {
    fn new(height: usize, width: usize, init: T) -> Self {
        Self {
            width,
            map: vec![init; height * width],
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
struct Pos {
    row: u32,
    col: u32,
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},{}]", self.row, self.col)
    }
}

impl Pos {
    const fn new(row: usize, col: usize) -> Self {
        Self {
            row: row as u32,
            col: col as u32,
        }
    }

    fn move_in(&self, direction: Direction) -> Self {
        match direction {
            Direction::North => self.nn(),
            Direction::East => self.ee(),
            Direction::South => self.ss(),
            Direction::West => self.ww(),
        }
    }

    fn move_opposite_of(self, direction: Direction) -> Self {
        match direction {
            Direction::North => self.ss(),
            Direction::East => self.ww(),
            Direction::South => self.nn(),
            Direction::West => self.ee(),
        }
    }

    fn move_in_orthogonal1(self, direction: Direction) -> Self {
        match direction {
            Direction::North => self.ee(),
            Direction::East => self.ss(),
            Direction::South => self.ww(),
            Direction::West => self.nn(),
        }
    }

    fn move_in_orthogonal2(self, direction: Direction) -> Self {
        match direction {
            Direction::North => self.ww(),
            Direction::East => self.nn(),
            Direction::South => self.ee(),
            Direction::West => self.ss(),
        }
    }

    #[allow(dead_code)]
    fn nn(self) -> Self {
        Self {
            row: self.row - 1,
            col: self.col,
        }
    }

    #[allow(dead_code)]
    fn ee(self) -> Self {
        Self {
            row: self.row,
            col: self.col + 1,
        }
    }

    #[allow(dead_code)]
    fn ss(self) -> Self {
        Self {
            row: self.row + 1,
            col: self.col,
        }
    }

    #[allow(dead_code)]
    fn ww(self) -> Self {
        Self {
            row: self.row,
            col: self.col - 1,
        }
    }
}

impl<T> ops::Index<Pos> for FlatMap<T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: Pos) -> &Self::Output {
        &self.map[index.row as usize * self.width + index.col as usize]
    }
}

impl<T> ops::IndexMut<Pos> for FlatMap<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.map[index.row as usize * self.width + index.col as usize]
    }
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut map = Map::parse(input);
    let total_cost = map.floodfill_solve();

    Some(Box::new(total_cost))
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    println!("{input}");
    let mut map = Map::parse(input);
    let total_cost = map.floodfill_solve_bulk();

    Some(Box::new(total_cost))
}

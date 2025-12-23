use std::{cmp, fmt, ops};

inventory::submit!(crate::days::DayModule::new(2024, 12).with_executors(
    crate::day_part_executors![part1],
    crate::day_part_executors![part2],
));

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

    fn floodfill_region_bulk_cost(
        &mut self,
        seed: Pos,
        to_search: &mut Vec<Pos>,
        horizontal_edges: &mut Vec<(Pos, HOrientation)>,
        vertical_edges: &mut Vec<(Pos, VOrientation)>,
    ) -> u64 {
        if self.mapped_plots[seed] {
            return 0;
        }

        let plot = self.plot(seed);
        let mut area = 0_u64;

        to_search.clear();
        to_search.push(seed);

        horizontal_edges.clear();
        vertical_edges.clear();

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
                    vertical_edges.push((next, VOrientation::Left));
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
                    {
                        let mut edge = next;
                        edge.col += 1;
                        vertical_edges.push((edge, VOrientation::Left));
                    }
                    break next.col + 1;
                }
            };

            let mut next = pos;
            let r_col = loop {
                if self.pos_on_east_edge(next) {
                    vertical_edges.push((next, VOrientation::Right));
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
                    {
                        let mut edge = next;
                        edge.col -= 1;
                        vertical_edges.push((edge, VOrientation::Right));
                    }
                    break next.col - 1;
                }
            };

            if self.pos_on_north_edge(pos) {
                for col in l_col..=r_col {
                    let next = Pos { row: pos.row, col };
                    horizontal_edges.push((next, HOrientation::Above));
                }
            } else {
                self.floodfill_scan_bulk(
                    plot,
                    pos.row - 1,
                    (l_col, r_col),
                    to_search,
                    horizontal_edges,
                    (pos.row, HOrientation::Above),
                );
            }

            if self.pos_on_south_edge(pos) {
                for col in l_col..=r_col {
                    let next = Pos { row: pos.row, col };
                    horizontal_edges.push((next, HOrientation::Below));
                }
            } else {
                self.floodfill_scan_bulk(
                    plot,
                    pos.row + 1,
                    (l_col, r_col),
                    to_search,
                    horizontal_edges,
                    (pos.row, HOrientation::Below),
                );
            }
        }

        horizontal_edges.sort_by(HorizontalEdgeOrdering::cmp);
        let mut horizontal_edges_iter = horizontal_edges.drain(..);
        let first_horizontal_edge = horizontal_edges_iter.next().unwrap();
        let horizontal_sides = horizontal_edges_iter
            .fold((1_u64, first_horizontal_edge), |mut acc, p| {
                if p.0.row > acc.1.0.row {
                    acc.0 += 1;
                    acc.1 = p;
                    return acc;
                }

                if p.1 > acc.1.1 {
                    acc.0 += 1;
                    acc.1.0 = p.0;
                    acc.1.1 = p.1;
                    return acc;
                }

                if p.0.col > acc.1.0.col + 1 {
                    acc.0 += 1;
                    acc.1.0.col = p.0.col;
                    return acc;
                }

                acc.1.0.col += 1;
                acc
            })
            .0;

        vertical_edges.sort_by(VerticalEdgeOrdering::cmp);
        let mut vertical_edges_iter = vertical_edges.drain(..);
        let first_vertical_edge = vertical_edges_iter.next().unwrap();
        let vertical_sides = vertical_edges_iter
            .fold((1_u64, first_vertical_edge), |mut acc, p| {
                if p.0.col > acc.1.0.col {
                    acc.0 += 1;
                    acc.1 = p;
                    return acc;
                }

                if p.1 > acc.1.1 {
                    acc.0 += 1;
                    acc.1.0 = p.0;
                    acc.1.1 = p.1;
                    return acc;
                }

                if p.0.row > acc.1.0.row + 1 {
                    acc.0 += 1;
                    acc.1.0.row = p.0.row;
                    return acc;
                }

                acc.1.0.row += 1;
                acc
            })
            .0;

        let sides = horizontal_sides + vertical_sides;

        area * sides
    }

    #[inline(always)]
    fn floodfill_scan_bulk(
        &self,
        plot: u8,
        row: u32,
        (l_col, r_col): (u32, u32),
        to_search: &mut Vec<Pos>,
        horizontal_edges: &mut Vec<(Pos, HOrientation)>,
        (horizontal_edge_row, horizontal_edge_orientation): (u32, HOrientation),
    ) {
        let mut next = Pos { row, col: l_col };
        let mut span_added = false;

        while next.col <= r_col {
            if self.plot(next) != plot {
                span_added = false;
                {
                    let mut edge = next;
                    edge.row = horizontal_edge_row;
                    horizontal_edges.push((edge, horizontal_edge_orientation));
                }
            } else if !span_added && !self.mapped_plots[next] {
                to_search.push(next);
                span_added = true;
            }

            next.col += 1;
        }
    }

    fn floodfill_solve_bulk(&mut self) -> u64 {
        let mut total_fence_cost = 0_u64;

        let cap = self.mapped_plots.map.len() / 4;
        let mut to_search = Vec::<Pos>::with_capacity(cap);
        let mut horizontal_edge_cache = Vec::<(Pos, HOrientation)>::with_capacity(cap);
        let mut vertical_edge_cache = Vec::<(Pos, VOrientation)>::with_capacity(cap);

        for r in 0..self.height {
            for c in 0..self.width {
                let seed = Pos::new(r, c);
                total_fence_cost += self.floodfill_region_bulk_cost(
                    seed,
                    &mut to_search,
                    &mut horizontal_edge_cache,
                    &mut vertical_edge_cache,
                );
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum HOrientation {
    Above,
    Below,
}

impl fmt::Debug for HOrientation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Above => write!(f, "U"),
            Self::Below => write!(f, "D"),
        }
    }
}

impl PartialOrd for HOrientation {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HOrientation {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (*self, *other) {
            (Self::Above, Self::Below) => cmp::Ordering::Less,
            (Self::Below, Self::Above) => cmp::Ordering::Greater,
            (_, _) => cmp::Ordering::Equal,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VOrientation {
    Left,
    Right,
}

impl fmt::Debug for VOrientation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Left => write!(f, "L"),
            Self::Right => write!(f, "R"),
        }
    }
}

impl PartialOrd for VOrientation {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VOrientation {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (*self, *other) {
            (Self::Left, Self::Right) => cmp::Ordering::Less,
            (Self::Right, Self::Left) => cmp::Ordering::Greater,
            (_, _) => cmp::Ordering::Equal,
        }
    }
}

trait HorizontalEdgeOrdering {
    fn cmp(a: &Self, b: &Self) -> cmp::Ordering;
}

impl HorizontalEdgeOrdering for (Pos, HOrientation) {
    fn cmp(a: &Self, b: &Self) -> cmp::Ordering {
        a.0.row
            .cmp(&b.0.row)
            .then(a.1.cmp(&b.1))
            .then(a.0.col.cmp(&b.0.col))
    }
}

trait VerticalEdgeOrdering {
    fn cmp(a: &Self, b: &Self) -> cmp::Ordering;
}

impl VerticalEdgeOrdering for (Pos, VOrientation) {
    fn cmp(a: &Self, b: &Self) -> cmp::Ordering {
        a.0.col
            .cmp(&b.0.col)
            .then(a.1.cmp(&b.1))
            .then(a.0.row.cmp(&b.0.row))
    }
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut map = Map::parse(input);
    let total_cost = map.floodfill_solve();

    Some(Box::new(total_cost))
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut map = Map::parse(input);
    let total_cost = map.floodfill_solve_bulk();

    Some(Box::new(total_cost))
}

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
    plot_region_ids: FlatMap<u32>,
}

impl<'input> Map<'input> {
    fn parse(input: &'input str) -> Self {
        let height = input.lines().count();
        let width = input.lines().next().unwrap().trim().len();

        let mut plots = Vec::with_capacity(height);
        plots.extend(input.lines().map(|line| line.as_bytes()));

        let plot_region_ids = FlatMap::new(height, width, u32::MAX);

        Self {
            height,
            width,
            plots,
            plot_region_ids,
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

    fn floodfill_region_cost(
        &mut self,
        seed: Pos,
        region_id: &mut u32,
        to_search: &mut Vec<Pos>,
    ) -> u64 {
        if self.plot_region_ids[seed] < u32::MAX {
            return 0;
        }

        let plot = self.plot(seed);
        let mut area = 0_u64;
        let mut perimeter = 0_u64;

        to_search.clear();
        to_search.push(seed);

        while let Some(pos) = to_search.pop() {
            {
                let pos_id = &mut self.plot_region_ids[pos];
                if *pos_id == *region_id {
                    continue;
                }
                *pos_id = *region_id;
            }

            area += 1;

            {
                if self.pos_on_west_edge(pos) {
                    perimeter += 1;
                } else {
                    let next = pos.ww();
                    if self.plot(next) == plot {
                        if self.plot_region_ids.cell_is_unmapped(next) {
                            to_search.push(next);
                        }
                    } else {
                        perimeter += 1;
                    }
                }
            }

            {
                if self.pos_on_east_edge(pos) {
                    perimeter += 1;
                } else {
                    let next = pos.ee();
                    if self.plot(next) == plot {
                        if self.plot_region_ids.cell_is_unmapped(next) {
                            to_search.push(next);
                        }
                    } else {
                        perimeter += 1;
                    }
                }
            }

            {
                if self.pos_on_north_edge(pos) {
                    perimeter += 1;
                } else {
                    let next = pos.nn();
                    if self.plot(next) == plot {
                        if self.plot_region_ids.cell_is_unmapped(next) {
                            to_search.push(next);
                        }
                    } else {
                        perimeter += 1;
                    }
                }
            }

            {
                if self.pos_on_south_edge(pos) {
                    perimeter += 1;
                } else {
                    let next = pos.ss();
                    if self.plot(next) == plot {
                        if self.plot_region_ids.cell_is_unmapped(next) {
                            to_search.push(next);
                        }
                    } else {
                        perimeter += 1;
                    }
                }
            }
        }

        *region_id += 1;
        area * perimeter
    }

    fn floodfill_solve(&mut self) -> u64 {
        let mut region_id = 0_u32;

        let mut total_fence_cost = 0_u64;

        let mut to_search = Vec::<Pos>::with_capacity(self.plot_region_ids.map.len() / 4);

        for r in 0..self.height {
            for c in 0..self.width {
                let seed = Pos::new(r, c);
                total_fence_cost +=
                    self.floodfill_region_cost(seed, &mut region_id, &mut to_search);
            }
        }

        total_fence_cost
    }
}

struct FlatMap<T> {
    width: usize,
    map: Vec<T>,
    sentinel: T,
}

impl<T: Clone + Copy + PartialEq + Eq> FlatMap<T> {
    fn new(height: usize, width: usize, sentinel: T) -> Self {
        Self {
            width,
            map: vec![sentinel; height * width],
            sentinel,
        }
    }

    fn cell_is_unmapped(&self, pos: Pos) -> bool {
        self[pos] == self.sentinel
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
    _ = input;

    None
}

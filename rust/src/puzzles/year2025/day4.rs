use adventofcode::{Grid1D, Grid2D, GridIndex1D, GridIndex2D, Pos};

inventory::submit!(crate::days::DayModule::new("2025", 4).with_executors(
    crate::day_part_executors![part1],
    crate::day_part_executors![part2],
));

struct PaperMap<'input> {
    width: usize,
    map: Vec<&'input [u8]>,
}

impl<'input> Grid2D for PaperMap<'input> {
    type Item = u8;

    fn grid_inner(&self) -> &[&[Self::Item]] {
        self.map.as_slice()
    }
}

impl<'input> PaperMap<'input> {
    fn parse(input: &'input str) -> Self {
        let map: Vec<_> = input.lines().map(str::as_bytes).collect();
        let width = map[0].len();
        PaperMap { width, map }
    }

    fn count_accessible_rolls(&self) -> usize {
        let mut accessible = 0;

        for r in 0..self.map.len() {
            for c in 0..self.map[0].len() {
                let pos = Pos::new(r, c);

                if *self.index_2d(pos) != b'@' {
                    continue;
                }

                let mut adjacent = 0_u32;

                let has_adj_n = pos.row > 0;
                let has_adj_e = (pos.col as usize) < self.width - 1;
                let has_adj_s = (pos.row as usize) < self.map.len() - 1;
                let has_adj_w = pos.col > 0;

                if has_adj_n && *self.index_2d(pos.nn()) == b'@' {
                    adjacent += 1;
                }
                if has_adj_n && has_adj_e && *self.index_2d(pos.ne()) == b'@' {
                    adjacent += 1;
                }
                if has_adj_e && *self.index_2d(pos.ee()) == b'@' {
                    adjacent += 1;
                }
                if has_adj_s && has_adj_e && *self.index_2d(pos.se()) == b'@' {
                    adjacent += 1;
                }
                if has_adj_s && *self.index_2d(pos.ss()) == b'@' {
                    adjacent += 1;
                }
                if has_adj_s && has_adj_w && *self.index_2d(pos.sw()) == b'@' {
                    adjacent += 1;
                }
                if has_adj_w && *self.index_2d(pos.ww()) == b'@' {
                    adjacent += 1;
                }
                if has_adj_n && has_adj_w && *self.index_2d(pos.nw()) == b'@' {
                    adjacent += 1;
                }
                if adjacent < 4 {
                    accessible += 1;
                }
            }
        }

        accessible
    }
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let map = PaperMap::parse(input);

    let accessible = map.count_accessible_rolls();

    Some(Box::new(accessible))
}

struct ForkliftManager {
    width: usize,
    height: usize,
    map: Vec<bool>,
}

impl Grid1D for ForkliftManager {
    type Item = bool;

    fn extent(&self) -> usize {
        self.width
    }

    fn grid_inner(&self) -> &[Self::Item] {
        self.map.as_slice()
    }

    fn grid_inner_mut(&mut self) -> &mut [Self::Item] {
        self.map.as_mut_slice()
    }
}

impl ForkliftManager {
    fn from_paper_map(paper_map: &PaperMap) -> Self {
        let mut map = Vec::with_capacity(paper_map.width * paper_map.map.len());
        map.extend(
            paper_map
                .map
                .iter()
                .flat_map(|row| row.iter().copied().map(|b| b == b'@')),
        );

        Self {
            width: paper_map.width,
            height: paper_map.map.len(),
            map,
        }
    }

    fn remove_accesible_rolls(&mut self) -> usize {
        let mut removed = 0;

        for r in 0..self.height {
            for c in 0..self.width {
                let pos = Pos::new(r, c);

                if !*self.index_1d(pos) {
                    continue;
                }

                let mut adjacent = 0_u32;

                let has_adj_n = pos.row > 0;
                let has_adj_e = (pos.col as usize) < self.width - 1;
                let has_adj_s = (pos.row as usize) < self.height - 1;
                let has_adj_w = pos.col > 0;

                if has_adj_n && *self.index_1d(pos.nn()) {
                    adjacent += 1;
                }
                if has_adj_n && has_adj_e && *self.index_1d(pos.ne()) {
                    adjacent += 1;
                }
                if has_adj_e && *self.index_1d(pos.ee()) {
                    adjacent += 1;
                }
                if has_adj_s && has_adj_e && *self.index_1d(pos.se()) {
                    adjacent += 1;
                }
                if has_adj_s && *self.index_1d(pos.ss()) {
                    adjacent += 1;
                }
                if has_adj_s && has_adj_w && *self.index_1d(pos.sw()) {
                    adjacent += 1;
                }
                if has_adj_w && *self.index_1d(pos.ww()) {
                    adjacent += 1;
                }
                if has_adj_n && has_adj_w && *self.index_1d(pos.nw()) {
                    adjacent += 1;
                }
                if adjacent < 4 {
                    *self.index_1d_mut(pos) = false;
                    removed += 1;
                }
            }
        }

        removed
    }

    fn remove_all_accesible_rolls(&mut self) -> usize {
        let mut removed = 0;

        loop {
            let r = self.remove_accesible_rolls();
            if r == 0 {
                break;
            }
            removed += r;
        }

        removed
    }
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let map = PaperMap::parse(input);
    let mut forklift_mngr = ForkliftManager::from_paper_map(&map);

    let moved = forklift_mngr.remove_all_accesible_rolls();

    Some(Box::new(moved))
}

use adventofcode::{Grid2D, GridIndex2D, Pos};

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

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    _ = input;

    None
}

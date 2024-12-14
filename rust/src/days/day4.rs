use std::{fmt, ops::Index};

use paste::paste;

#[derive(Clone, Copy, Debug)]
struct Pos {
    row: usize,
    col: usize,
}

impl Pos {
    fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{},{}]", self.row, self.col)
    }
}

macro_rules! pos_ordinals {
    ($( $x:literal ),+ $(,)?) => {
        $(
            paste! {
                fn [<nn $x>] (self) -> Self {
                    Self {
                        row: self.row - $x,
                        col: self.col,
                    }
                }

                fn [<ne $x>] (self) -> Self {
                    Self {
                        row: self.row - $x,
                        col: self.col + $x,
                    }
                }

                fn [<ee $x>] (self) -> Self {
                    Self {
                        row: self.row,
                        col: self.col + $x,
                    }
                }

                fn [<se $x>] (self) -> Self {
                    Self {
                        row: self.row + $x,
                        col: self.col + $x,
                    }
                }

                fn [<ss $x>] (self) -> Self {
                    Self {
                        row: self.row + $x,
                        col: self.col,
                    }
                }

                fn [<sw $x>] (self) -> Self {
                    Self {
                        row: self.row + $x,
                        col: self.col - $x,
                    }
                }

                fn [<ww $x>] (self) -> Self {
                    Self {
                        row: self.row,
                        col: self.col - $x,
                    }
                }

                fn [<nw $x>] (self) -> Self {
                    Self {
                        row: self.row - $x,
                        col: self.col - $x,
                    }
                }
            }
        )+
    };
}

impl Pos {
    pos_ordinals![
        1, // `M`
        2, // `A`
        3, // `S`
    ];
}

struct WordSearch<'input> {
    height: usize,
    width: usize,
    table: Vec<&'input str>,
}

impl<'input> WordSearch<'input> {
    fn new(input: &'input str) -> Self {
        let table = input.lines().collect::<Vec<_>>();
        Self {
            height: table.len(),
            width: table[0].len(),
            table,
        }
    }

    fn x_positions(&self) -> impl Iterator<Item = Pos> {
        self.table.iter().enumerate().flat_map(|(r, &line)| {
            line.bytes()
                .enumerate()
                .filter_map(move |(c, b)| (b == b'X').then_some(Pos::new(r, c)))
        })
    }

    fn probe_xmas(&self, x: Pos) -> u32 {
        let mut count = 0;

        let can_probe_e = x.col < self.width - 3;
        let can_probe_s = x.row < self.height - 3;
        let can_probe_w = x.col > 2;
        let can_probe_n = x.row > 2;

        const MAS: [u8; 3] = [b'M', b'A', b'S'];

        if can_probe_n {
            if [self[x.nn1()], self[x.nn2()], self[x.nn3()]] == MAS {
                count += 1;
            }
            if can_probe_w && [self[x.nw1()], self[x.nw2()], self[x.nw3()]] == MAS {
                count += 1;
            }
            if can_probe_e && [self[x.ne1()], self[x.ne2()], self[x.ne3()]] == MAS {
                count += 1;
            }
        }

        if can_probe_s {
            if [self[x.ss1()], self[x.ss2()], self[x.ss3()]] == MAS {
                count += 1;
            }
            if can_probe_w && [self[x.sw1()], self[x.sw2()], self[x.sw3()]] == MAS {
                count += 1;
            }
            if can_probe_e && [self[x.se1()], self[x.se2()], self[x.se3()]] == MAS {
                count += 1;
            }
        }

        if can_probe_e && [self[x.ee1()], self[x.ee2()], self[x.ee3()]] == MAS {
            count += 1;
        }

        if can_probe_w && [self[x.ww1()], self[x.ww2()], self[x.ww3()]] == MAS {
            count += 1;
        }

        count
    }

    fn count_xmas(&self) -> u32 {
        self.x_positions().map(|x_pos| self.probe_xmas(x_pos)).sum()
    }
}

impl Index<Pos> for WordSearch<'_> {
    type Output = u8;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.table[index.row].as_bytes()[index.col]
    }
}

pub(super) fn part1(input: &str) -> Box<dyn std::fmt::Display> {
    let word_search = WordSearch::new(input);

    let answer = word_search.count_xmas();

    Box::new(answer)
}

pub(super) fn part2(input: &str) -> Box<dyn std::fmt::Display> {
    _ = input;

    Box::new("_")
}

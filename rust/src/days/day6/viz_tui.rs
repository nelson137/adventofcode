#![allow(dead_code)]

use super::{Cell, Map};

impl Map {
    pub(super) fn visualize(&self) -> anyhow::Result<()> {
        use crossterm::{
            cursor::{MoveTo, MoveToNextLine},
            execute, queue,
            style::{PrintStyledContent, Stylize},
            terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
        };

        execute!(
            std::io::stdout(),
            EnterAlternateScreen,
            MoveTo(0, 0),
            Clear(ClearType::All)
        )?;

        fn visualize_grid(grid: &[Vec<Cell>]) -> anyhow::Result<()> {
            let mut stdout = std::io::stdout();

            const CHUNK_SIZE: usize = 3;
            mod c {
                use crossterm::style::Color;

                // pub const EEE_EEE: &str = " ";
                // pub const OEE_OEE: &str = "\u{1fb02}";
                // pub const EOE_EOE: &str = "\u{1fb0b}";
                // pub const OOE_OOE: &str = "\u{1fb0e}";
                // pub const EEO_EEO: &str = "\u{1fb2d}";
                // pub const OEO_OEO: &str = "\u{1fb30}";
                // pub const EOO_EOO: &str = "\u{1fb39}";
                // pub const OOO_OOO: &str = "\u{2588}";

                pub const O_COLOR: Color = Color::DarkRed;

                pub const EE_EE_EE: &str = " ";
                pub const OE_OE_OE: &str = "▌";
                pub const EO_EO_EO: &str = "▐";
                pub const OO_OO_OO: &str = "█";

                // U+1fb0_
                pub const OE_EE_EE: &str = "\u{1fb00}";
                pub const EO_EE_EE: &str = "\u{1fb01}";
                pub const OO_EE_EE: &str = "\u{1fb02}";
                pub const EE_OE_EE: &str = "\u{1fb03}";
                pub const OE_OE_EE: &str = "\u{1fb04}";
                pub const EO_OE_EE: &str = "\u{1fb05}";
                pub const OO_OE_EE: &str = "\u{1fb06}";
                pub const EE_EO_EE: &str = "\u{1fb07}";
                pub const OE_EO_EE: &str = "\u{1fb08}";
                pub const EO_EO_EE: &str = "\u{1fb09}";
                pub const OO_EO_EE: &str = "\u{1fb0a}";
                pub const EE_OO_EE: &str = "\u{1fb0b}";
                pub const OE_OO_EE: &str = "\u{1fb0c}";
                pub const EO_OO_EE: &str = "\u{1fb0d}";
                pub const OO_OO_EE: &str = "\u{1fb0e}";
                pub const EE_EE_OE: &str = "\u{1fb0f}";

                // U+1fb1_
                pub const OE_EE_OE: &str = "\u{1fb10}";
                pub const EO_EE_OE: &str = "\u{1fb11}";
                pub const OO_EE_OE: &str = "\u{1fb12}";
                pub const EE_OE_OE: &str = "\u{1fb13}";
                pub const EO_OE_OE: &str = "\u{1fb14}";
                pub const OO_OE_OE: &str = "\u{1fb15}";
                pub const EE_EO_OE: &str = "\u{1fb16}";
                pub const OE_EO_OE: &str = "\u{1fb17}";
                pub const EO_EO_OE: &str = "\u{1fb18}";
                pub const OO_EO_OE: &str = "\u{1fb19}";
                pub const EE_OO_OE: &str = "\u{1fb1a}";
                pub const OE_OO_OE: &str = "\u{1fb1b}";
                pub const EO_OO_OE: &str = "\u{1fb1c}";
                pub const OO_OO_OE: &str = "\u{1fb1d}";
                pub const EE_EE_EO: &str = "\u{1fb1e}";
                pub const OE_EE_EO: &str = "\u{1fb1f}";

                // U+1fb2_
                pub const EO_EE_EO: &str = "\u{1fb20}";
                pub const OO_EE_EO: &str = "\u{1fb21}";
                pub const EE_OE_EO: &str = "\u{1fb22}";
                pub const OE_OE_EO: &str = "\u{1fb23}";
                pub const EO_OE_EO: &str = "\u{1fb24}";
                pub const OO_OE_EO: &str = "\u{1fb25}";
                pub const EE_EO_EO: &str = "\u{1fb26}";
                pub const OE_EO_EO: &str = "\u{1fb27}";
                pub const OO_EO_EO: &str = "\u{1fb28}";
                pub const EE_OO_EO: &str = "\u{1fb29}";
                pub const OE_OO_EO: &str = "\u{1fb2a}";
                pub const EO_OO_EO: &str = "\u{1fb2b}";
                pub const OO_OO_EO: &str = "\u{1fb2c}";
                pub const EE_EE_OO: &str = "\u{1fb2d}";
                pub const OE_EE_OO: &str = "\u{1fb2e}";
                pub const EO_EE_OO: &str = "\u{1fb2f}";

                // U+1fb3_
                pub const OO_EE_OO: &str = "\u{1fb30}";
                pub const EE_OE_OO: &str = "\u{1fb31}";
                pub const OE_OE_OO: &str = "\u{1fb32}";
                pub const EO_OE_OO: &str = "\u{1fb33}";
                pub const OO_OE_OO: &str = "\u{1fb34}";
                pub const EE_EO_OO: &str = "\u{1fb35}";
                pub const OE_EO_OO: &str = "\u{1fb36}";
                pub const EO_EO_OO: &str = "\u{1fb37}";
                pub const OO_EO_OO: &str = "\u{1fb38}";
                pub const EE_OO_OO: &str = "\u{1fb39}";
                pub const OE_OO_OO: &str = "\u{1fb3a}";
                pub const EO_OO_OO: &str = "\u{1fb3b}";

                // pub const xx_xx_xx: &str = "\u{1fb3c}";
                // pub const xx_xx_xx: &str = "\u{1fb3d}";
                // pub const xx_xx_xx: &str = "\u{1fb3e}";
                // pub const xx_xx_xx: &str = "\u{1fb3f}";
            }

            for (row_pair_i, row_pair) in grid.chunks(CHUNK_SIZE).enumerate() {
                use Cell::{Empty as E, EmptyVisited as V, Obstacle as O};

                let r1 = row_pair_i * CHUNK_SIZE;
                let r2 = row_pair_i * CHUNK_SIZE + 1;
                let r3 = row_pair_i * CHUNK_SIZE + 2;

                fn x((l, r): (&Cell, &Cell)) -> &'static str {
                    match (l, r) {
                        (E, E) => "EE",
                        (V, E) => "VE",
                        (O, E) => "OE",
                        (E, V) => "EV",
                        (V, V) => "VV",
                        (O, V) => "OV",
                        (E, O) => "EO",
                        (V, O) => "VO",
                        (O, O) => "OO",
                    }
                }

                match row_pair {
                    [row1, row2, row3] => {
                        let mut cells_iter = row1.iter().zip(row2.iter().zip(row3)).enumerate();

                        while let (Some(left), Some(right)) = (cells_iter.next(), cells_iter.next())
                        {
                            let (cl, (cell_1l, (cell_2l, cell_3l))) = left;
                            let (cr, (cell_1r, (cell_2r, cell_3r))) = right;
                            let cmd = match (
                                (cell_1l, cell_1r),
                                (cell_2l, cell_2r),
                                (cell_3l, cell_3r),
                            ) {
                                ((E, E), (E, E), (E, E)) => c::EE_EE_EE.with(c::O_COLOR),
                                ((O, E), (E, E), (E, E)) => c::OE_EE_EE.with(c::O_COLOR),
                                ((E, O), (E, E), (E, E)) => c::EO_EE_EE.with(c::O_COLOR),
                                ((O, O), (E, E), (E, E)) => c::OO_EE_EE.with(c::O_COLOR),
                                ((E, E), (O, E), (E, E)) => c::EE_OE_EE.with(c::O_COLOR),
                                ((O, E), (O, E), (E, E)) => c::OE_OE_EE.with(c::O_COLOR),
                                ((E, O), (O, E), (E, E)) => c::EO_OE_EE.with(c::O_COLOR),
                                ((O, O), (O, E), (E, E)) => c::OO_OE_EE.with(c::O_COLOR),
                                ((E, E), (E, O), (E, E)) => c::EE_EO_EE.with(c::O_COLOR),
                                ((O, E), (E, O), (E, E)) => c::OE_EO_EE.with(c::O_COLOR),
                                ((E, O), (E, O), (E, E)) => c::EO_EO_EE.with(c::O_COLOR),
                                ((O, O), (E, O), (E, E)) => c::OO_EO_EE.with(c::O_COLOR),
                                ((E, E), (O, O), (E, E)) => c::EE_OO_EE.with(c::O_COLOR),
                                ((O, E), (O, O), (E, E)) => c::OE_OO_EE.with(c::O_COLOR),
                                ((E, O), (O, O), (E, E)) => c::EO_OO_EE.with(c::O_COLOR),
                                ((O, O), (O, O), (E, E)) => c::OO_OO_EE.with(c::O_COLOR),
                                ((E, E), (E, E), (O, E)) => c::EE_EE_OE.with(c::O_COLOR),
                                ((O, E), (E, E), (O, E)) => c::OE_EE_OE.with(c::O_COLOR),
                                ((E, O), (E, E), (O, E)) => c::EO_EE_OE.with(c::O_COLOR),
                                ((O, O), (E, E), (O, E)) => c::OO_EE_OE.with(c::O_COLOR),
                                ((E, E), (O, E), (O, E)) => c::EE_OE_OE.with(c::O_COLOR),
                                ((O, E), (O, E), (O, E)) => c::OE_OE_OE.with(c::O_COLOR),
                                ((E, O), (O, E), (O, E)) => c::EO_OE_OE.with(c::O_COLOR),
                                ((O, O), (O, E), (O, E)) => c::OO_OE_OE.with(c::O_COLOR),
                                ((E, E), (E, O), (O, E)) => c::EE_EO_OE.with(c::O_COLOR),
                                ((O, E), (E, O), (O, E)) => c::OE_EO_OE.with(c::O_COLOR),
                                ((E, O), (E, O), (O, E)) => c::EO_EO_OE.with(c::O_COLOR),
                                ((O, O), (E, O), (O, E)) => c::OO_EO_OE.with(c::O_COLOR),
                                ((E, E), (O, O), (O, E)) => c::EE_OO_OE.with(c::O_COLOR),
                                ((O, E), (O, O), (O, E)) => c::OE_OO_OE.with(c::O_COLOR),
                                ((E, O), (O, O), (O, E)) => c::EO_OO_OE.with(c::O_COLOR),
                                ((O, O), (O, O), (O, E)) => c::OO_OO_OE.with(c::O_COLOR),
                                ((E, E), (E, E), (E, O)) => c::EE_EE_EO.with(c::O_COLOR),
                                ((O, E), (E, E), (E, O)) => c::OE_EE_EO.with(c::O_COLOR),
                                ((E, O), (E, E), (E, O)) => c::EO_EE_EO.with(c::O_COLOR),
                                ((O, O), (E, E), (E, O)) => c::OO_EE_EO.with(c::O_COLOR),
                                ((E, E), (O, E), (E, O)) => c::EE_OE_EO.with(c::O_COLOR),
                                ((O, E), (O, E), (E, O)) => c::OE_OE_EO.with(c::O_COLOR),
                                ((E, O), (O, E), (E, O)) => c::EO_OE_EO.with(c::O_COLOR),
                                ((O, O), (O, E), (E, O)) => c::OO_OE_EO.with(c::O_COLOR),
                                ((E, E), (E, O), (E, O)) => c::EE_EO_EO.with(c::O_COLOR),
                                ((O, E), (E, O), (E, O)) => c::OE_EO_EO.with(c::O_COLOR),
                                ((E, O), (E, O), (E, O)) => c::EO_EO_EO.with(c::O_COLOR),
                                ((O, O), (E, O), (E, O)) => c::OO_EO_EO.with(c::O_COLOR),
                                ((E, E), (O, O), (E, O)) => c::EE_OO_EO.with(c::O_COLOR),
                                ((O, E), (O, O), (E, O)) => c::OE_OO_EO.with(c::O_COLOR),
                                ((E, O), (O, O), (E, O)) => c::EO_OO_EO.with(c::O_COLOR),
                                ((O, O), (O, O), (E, O)) => c::OO_OO_EO.with(c::O_COLOR),
                                ((E, E), (E, E), (O, O)) => c::EE_EE_OO.with(c::O_COLOR),
                                ((O, E), (E, E), (O, O)) => c::OE_EE_OO.with(c::O_COLOR),
                                ((E, O), (E, E), (O, O)) => c::EO_EE_OO.with(c::O_COLOR),
                                ((O, O), (E, E), (O, O)) => c::OO_EE_OO.with(c::O_COLOR),
                                ((E, E), (O, E), (O, O)) => c::EE_OE_OO.with(c::O_COLOR),
                                ((O, E), (O, E), (O, O)) => c::OE_OE_OO.with(c::O_COLOR),
                                ((E, O), (O, E), (O, O)) => c::EO_OE_OO.with(c::O_COLOR),
                                ((O, O), (O, E), (O, O)) => c::OO_OE_OO.with(c::O_COLOR),
                                ((E, E), (E, O), (O, O)) => c::EE_EO_OO.with(c::O_COLOR),
                                ((O, E), (E, O), (O, O)) => c::OE_EO_OO.with(c::O_COLOR),
                                ((E, O), (E, O), (O, O)) => c::EO_EO_OO.with(c::O_COLOR),
                                ((O, O), (E, O), (O, O)) => c::OO_EO_OO.with(c::O_COLOR),
                                ((E, E), (O, O), (O, O)) => c::EE_OO_OO.with(c::O_COLOR),
                                ((O, E), (O, O), (O, O)) => c::OE_OO_OO.with(c::O_COLOR),
                                ((E, O), (O, O), (O, O)) => c::EO_OO_OO.with(c::O_COLOR),
                                ((O, O), (O, O), (O, O)) => c::OO_OO_OO.with(c::O_COLOR),

                                ((E, V), (E, E), (E, E)) => c::EO_EE_EE.with(c::O_COLOR),

                                (top, mid, bot) => todo!(
                                    "TOP: [{r1},{cl}-{cr}] {} / MID: [{r2},{cl}-{cr}] {} / BOT: [{r3},{cl}-{cr}] {}",
                                    x(top),
                                    x(mid),
                                    x(bot)
                                ),
                            };

                            // let border = PrintStyledContent("▌".white());
                            queue!(stdout, PrintStyledContent(cmd.on_black()))?;
                        }

                        // for (c, ((top_cell, mid_cell), bot_cell)) in
                        //     row1.iter().zip(row2).zip(row3).enumerate()
                        // {
                        //     use Cell::*;
                        //     let cmd = match (top_cell, mid_cell, bot_cell) {
                        //         (Empty, Empty, Empty) => c::EEE_EEE.dark_red(),
                        //         (Obstacle, Empty, Empty) => c::OEE_OEE.dark_red(),
                        //         (Empty, Obstacle, Empty) => c::EOE_EOE.dark_red(),
                        //         (Obstacle, Obstacle, Empty) => c::OOE_OOE.dark_red(),
                        //         (Empty, Empty, Obstacle) => c::EEO_EEO.dark_red(),
                        //         (Obstacle, Empty, Obstacle) => c::OEO_OEO.dark_red(),
                        //         (Empty, Obstacle, Obstacle) => c::EOO_EOO.dark_red(),
                        //         (Obstacle, Obstacle, Obstacle) => c::OOO_OOO.dark_red(),
                        //         // (Cell::EmptyVisited(_), Cell::Empty) => "▀".grey(),
                        //         // (Cell::Empty, Cell::EmptyVisited(_)) => "▄".grey(),
                        //         _ => todo!(
                        //             "[{r1},{c}] {top_cell:?} / [{r2},{c}] {mid_cell:?} / [{r3},{c}] {bot_cell:?}"
                        //         ),
                        //     };
                        //     // let border = PrintStyledContent("▌".white());
                        //     queue!(stdout, PrintStyledContent(cmd.on_black()))?;
                        // }

                        queue!(stdout, MoveToNextLine(1))?;
                    }

                    [row1, row2] => {
                        let mut cells_iter = row1.iter().zip(row2).enumerate();

                        while let (Some(left), Some(right)) = (cells_iter.next(), cells_iter.next())
                        {
                            let (cl, (cell_1l, cell_2l)) = left;
                            let (cr, (cell_1r, cell_2r)) = right;
                            let cmd = match ((cell_1l, cell_1r), (cell_2l, cell_2r)) {
                                ((E, E), (E, E)) => c::EE_EE_EE.with(c::O_COLOR),
                                ((O, E), (E, E)) => c::OE_EE_EE.with(c::O_COLOR),
                                ((E, O), (E, E)) => c::EO_EE_EE.with(c::O_COLOR),
                                ((O, O), (E, E)) => c::OO_EE_EE.with(c::O_COLOR),
                                ((E, E), (O, E)) => c::EE_OE_EE.with(c::O_COLOR),
                                ((O, E), (O, E)) => c::OE_OE_EE.with(c::O_COLOR),
                                ((E, O), (O, E)) => c::EO_OE_EE.with(c::O_COLOR),
                                ((O, O), (O, E)) => c::OO_OE_EE.with(c::O_COLOR),
                                ((E, E), (E, O)) => c::EE_EO_EE.with(c::O_COLOR),
                                ((O, E), (E, O)) => c::OE_EO_EE.with(c::O_COLOR),
                                ((E, O), (E, O)) => c::EO_EO_EE.with(c::O_COLOR),
                                ((O, O), (E, O)) => c::OO_EO_EE.with(c::O_COLOR),
                                ((E, E), (O, O)) => c::EE_OO_EE.with(c::O_COLOR),
                                ((O, E), (O, O)) => c::OE_OO_EE.with(c::O_COLOR),
                                ((E, O), (O, O)) => c::EO_OO_EE.with(c::O_COLOR),
                                ((O, O), (O, O)) => c::OO_OO_EE.with(c::O_COLOR),

                                (top, mid) => todo!(
                                    "TOP: [{r1},{cl}-{cr}] {} / MID: [{r2},{cl}-{cr}] {}",
                                    x(top),
                                    x(mid),
                                ),
                            };

                            queue!(stdout, PrintStyledContent(cmd.on_black()))?;
                        }

                        queue!(stdout, MoveToNextLine(1))?;
                    }

                    [row] => {
                        let mut cells_iter = row.iter().enumerate();

                        while let (Some(left), Some(right)) = (cells_iter.next(), cells_iter.next())
                        {
                            let (cl, cell_l) = left;
                            let (cr, cell_r) = right;
                            let cmd = match (cell_l, cell_r) {
                                (E, E) => c::EE_EE_EE.with(c::O_COLOR),
                                (O, E) => c::OE_EE_EE.with(c::O_COLOR),
                                (E, O) => c::EO_EE_EE.with(c::O_COLOR),
                                (O, O) => c::OO_EE_EE.with(c::O_COLOR),

                                top => todo!("TOP: [{r1},{cl}-{cr}] {}", x(top)),
                            };

                            queue!(stdout, PrintStyledContent(cmd.on_black()))?;
                        }

                        queue!(stdout, MoveToNextLine(1))?;
                    }

                    _ => unreachable!(),
                }
            }

            // for _ in 0..=grid[0].len() {
            //     queue!(stdout, "")
            // }

            std::io::Write::flush(&mut stdout)?;

            Ok(())
        }

        visualize_grid(self.grid.as_slice())?;
        std::io::stdin().read_line(&mut String::new())?;

        // loop {}

        execute!(std::io::stdout(), LeaveAlternateScreen)?;

        Ok(())
    }
}

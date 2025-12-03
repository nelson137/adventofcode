use std::{
    collections::{HashSet, VecDeque},
    fmt::Write,
    time::{Duration, Instant},
};

use anyhow::Result;
use crossterm::event;
use ratatui::{
    DefaultTerminal, Frame, layout,
    style::{Style, Stylize},
    symbols, text, widgets,
};

use crate::days::day16 as day;

pub fn run(maze: &day::Maze) {
    let result = {
        let mut terminal = ratatui::init();
        App::new(maze).run(&mut terminal)
    };

    ratatui::restore();

    result.unwrap()
}

struct App<'maze> {
    // Maze
    maze: &'maze day::Maze<'maze>,
    // Solution algorithm state metadata
    steps: u32,
    logs: VecDeque<String>,
    // Solution algorithm state
    discovered: day::GridVec<bool>,
    open_set: day::MinHeap<day::ScoredNode>,
    dist: day::GridVec<[u64; 4]>,
    preceding: day::GridVec<HashSet<day::ScoredNode>>,
    solution: Option<day::ScoredNode>,
    solution_path: HashSet<day::Pos>,
    // UI state
    maze_table_state: widgets::TableState,
    maze_scroll_state: widgets::ScrollbarState,
    logs_scroll_offset: usize,
    logs_scroll_state: widgets::ScrollbarState,
}

impl<'maze> App<'maze> {
    fn new<'m: 'maze>(maze: &'m day::Maze<'m>) -> Self {
        let selected = Some(maze.start_pos.into());

        let start = day::Node::new(maze.start_pos, day::Direction::default());

        let mut open_set = day::MinHeap::<day::ScoredNode>::new();
        open_set.push(day::ScoredNode::new(start));

        let dist = day::GridVec::new(maze.width(), maze.height(), [u64::MAX; 4]);

        let preceding = day::GridVec::new(maze.width(), maze.height(), Default::default());

        let mut this = Self {
            maze,
            steps: 0,
            logs: VecDeque::new(),
            discovered: day::GridVec::new(maze.width(), maze.height(), false),
            open_set,
            dist,
            preceding,
            solution: None,
            solution_path: HashSet::new(),
            maze_table_state: widgets::TableState::new().with_selected_cell(selected),
            maze_scroll_state: {
                let mut state = widgets::ScrollbarState::new(maze.height());
                state.last();
                state
            },
            logs_scroll_offset: 0,
            logs_scroll_state: widgets::ScrollbarState::default(),
        };

        // this.steps = 215; //413; // XXX: test1: before end pos discovered
        // this.steps = 285; // XXX: test2: before end pos discovered
        // this.steps = 279; // XXX: test3: before end pos discovered
        // this.steps = 932393; // XXX: input ??: before end pos discovered
        // this.steps = 24074; // XXX: input2: before end pos discovered
        this.steps = 15983; // XXX: input2: before end pos discovered (with backtracking prevention)
        this.steps = 3000;

        for i in 1..=this.steps {
            let current_solution = this.step();
            match current_solution {
                Some(solution) if this.solution.is_none() => this.log(format!(
                    "Found solution during startup: step={} cost={}",
                    i, solution.f_score
                )),
                _ => {}
            }
            this.solution = this.solution.or(current_solution);
        }

        this
    }

    fn log(&mut self, log: impl Into<String>) {
        while self.logs.len() >= 4096 {
            self.logs.pop_front();
        }
        self.logs.push_back(log.into());
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        terminal.draw(|frame| self.draw(frame))?;

        loop {
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                let evt = event::read()?;
                if let event::Event::Key(key) = evt {
                    match key.code {
                        // Quit
                        event::KeyCode::Char('q') => break,
                        // Step algorithm
                        event::KeyCode::Char(' ') => {
                            self.steps += 1;
                            let current_solution = self.step();
                            match current_solution {
                                Some(solution) if self.solution.is_none() => {
                                    self.log(format!("SOLVED {}", solution.f_score));
                                    self.solve_path(solution);
                                }
                                _ => {}
                            }
                            self.solution = self.solution.or(current_solution);
                        }

                        //
                        // Cursor
                        //

                        // Down
                        event::KeyCode::Down => self.scroll_maze_down(1),
                        event::KeyCode::Char('j') => self.scroll_maze_down(1),
                        event::KeyCode::Char('J') => self.scroll_maze_down(10),
                        // Up
                        event::KeyCode::Up => self.scroll_maze_up(1),
                        event::KeyCode::Char('k') => self.scroll_maze_up(1),
                        event::KeyCode::Char('K') => self.scroll_maze_up(10),
                        // Left
                        event::KeyCode::Left => self.scroll_maze_left(1),
                        event::KeyCode::Char('h') => self.scroll_maze_left(1),
                        event::KeyCode::Char('H') => self.scroll_maze_left(10),
                        // Right
                        event::KeyCode::Right => self.scroll_maze_right(1),
                        event::KeyCode::Char('l') => self.scroll_maze_right(1),
                        event::KeyCode::Char('L') => self.scroll_maze_right(10),
                        // Goto start
                        event::KeyCode::Char('s') => {
                            self.maze_table_state
                                .select_cell(Some(self.maze.start_pos.into()));
                            self.maze_scroll_state.last();
                        }
                        // Goto end
                        event::KeyCode::Char('e') => {
                            self.maze_table_state
                                .select_cell(Some(self.maze.end_pos.into()));
                            self.maze_scroll_state.first();
                        }

                        //
                        // Logs
                        //

                        // Down
                        event::KeyCode::PageDown => {
                            let amount = if key.modifiers.intersects(event::KeyModifiers::SHIFT) {
                                10
                            } else {
                                1
                            };
                            self.scroll_logs_down(amount);
                        }
                        // Up
                        event::KeyCode::PageUp => {
                            let amount = if key.modifiers.intersects(event::KeyModifiers::SHIFT) {
                                10
                            } else {
                                1
                            };
                            self.scroll_logs_up(amount);
                        }
                        // Start
                        event::KeyCode::Home => {
                            self.logs_scroll_offset = 0;
                            self.logs_scroll_state = self.logs_scroll_state.position(0);
                        }
                        // End
                        event::KeyCode::End => {
                            let offset = self.logs.len().saturating_sub(1);
                            self.logs_scroll_offset = offset;
                            self.logs_scroll_state = self.logs_scroll_state.position(offset);
                        }
                        _ => {}
                    }
                }
                if let event::Event::Mouse(mouse) = evt {
                    match mouse.kind {
                        event::MouseEventKind::ScrollDown => self.scroll_maze_down(1),
                        event::MouseEventKind::ScrollUp => self.scroll_maze_up(1),
                        event::MouseEventKind::ScrollLeft => self.scroll_maze_left(1),
                        event::MouseEventKind::ScrollRight => self.scroll_maze_right(1),
                        _ => {}
                    }
                }
            }

            terminal.draw(|frame| self.draw(frame))?;

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn scroll_maze_up(&mut self, amount: u32) {
        for _ in 0..amount {
            self.maze_table_state.select_previous();
            self.maze_scroll_state.prev();
        }
    }

    fn scroll_maze_down(&mut self, amount: u32) {
        for _ in 0..amount {
            self.maze_table_state.select_next();
            self.maze_scroll_state.next();
        }
    }

    fn scroll_maze_left(&mut self, amount: u32) {
        for _ in 0..amount {
            self.maze_table_state.select_previous_column();
        }
    }

    fn scroll_maze_right(&mut self, amount: u32) {
        for _ in 0..amount {
            self.maze_table_state.select_next_column();
        }
    }

    fn scroll_logs_up(&mut self, amount: usize) {
        let offset = self.logs_scroll_offset.saturating_sub(amount);
        self.logs_scroll_offset = offset;
        self.logs_scroll_state = self.logs_scroll_state.position(offset);
    }

    fn scroll_logs_down(&mut self, amount: usize) {
        let offset = self.logs_scroll_offset.saturating_add(amount);
        self.logs_scroll_offset = offset;
        self.logs_scroll_state = self.logs_scroll_state.position(offset);
    }

    fn step(&mut self) -> Option<day::ScoredNode> {
        if let Some(current) = self.open_set.pop() {
            if current.node.pos == self.maze.end_pos {
                return Some(current);
            }

            const TARGET_POS: day::Pos = day::Pos::new(51, 133);
            let is_target = |n: &day::ScoredNode| {
                n.node.pos == TARGET_POS && n.node.dir == day::Direction::North
            };

            {
                let next = current.forward();
                // let _next_pos_row = next.node.pos.row; // XXX
                // let _next_pos_col = next.node.pos.col; // XXX
                // let _next_dir = next.node.dir; // XXX
                let next_dist = self.dist[next];

                if self.maze[next.node.pos] != b'#' && next.f_score <= next_dist {
                    if is_target(&next) {
                        self.log(format!(
                            "{} {} -> {} ({} -> {})",
                            current.node.pos,
                            current.node.dir,
                            next.node.pos,
                            next_dist,
                            next.f_score
                        ));
                    }
                    self.dist[next] = next.f_score;
                    if is_target(&next) {
                        self.log(format!("  before: {:?}", &self.preceding[next.node.pos]));
                    }
                    self.preceding[next.node.pos].retain(|p| p.f_score <= next.f_score);
                    if is_target(&next) {
                        self.log(format!("  retain: {:?}", &self.preceding[next.node.pos]));
                    }
                    if !self.discovered[next.node.pos] {
                        self.discovered[next.node.pos] = true;
                        #[allow(clippy::collapsible_if)]
                        if self.preceding[next.node.pos].insert(current) {
                            self.open_set.push(next);
                        }
                    }
                    if is_target(&next) {
                        self.log(format!("  after:  {:?}", &self.preceding[next.node.pos]));
                    }
                }
            }

            {
                let next = current.rotate_cw();
                let next_forward_pos = next.forward_pos();
                let next_dist = self.dist[next];

                if self.maze[next_forward_pos] != b'#' && next.f_score <= next_dist {
                    if is_target(&next) {
                        self.log(format!(
                            "{} {} -> {} ({} -> {})",
                            current.node.pos,
                            current.node.dir,
                            next.node.dir,
                            next_dist,
                            next.f_score
                        ));
                    }
                    self.dist[next] = next.f_score;
                    if is_target(&next) {
                        self.log(format!("  before: {:?}", &self.preceding[next.node.pos]));
                    }
                    self.preceding[next.node.pos].retain(|p| p.f_score <= next.f_score);
                    if is_target(&next) {
                        self.log(format!("  retain: {:?}", &self.preceding[next.node.pos]));
                    }
                    if self.preceding[next.node.pos].insert(current) {
                        self.open_set.push(next);
                    }
                    if is_target(&next) {
                        self.log(format!("  after:  {:?}", &self.preceding[next.node.pos]));
                    }
                }
            }

            {
                let next = current.rotate_ccw();
                let next_forward_pos = next.forward_pos();
                let next_dist = self.dist[next];

                if self.maze[next_forward_pos] != b'#' && next.f_score <= next_dist {
                    if is_target(&next) {
                        self.log(format!(
                            "{} {} -> {} ({} -> {})",
                            current.node.pos,
                            current.node.dir,
                            next.node.dir,
                            next_dist,
                            next.f_score
                        ));
                    }
                    self.dist[next] = next.f_score;
                    if is_target(&next) {
                        self.log(format!("  before: {:?}", &self.preceding[next.node.pos]));
                    }
                    self.preceding[next.node.pos].retain(|p| p.f_score <= next.f_score);
                    if is_target(&next) {
                        self.log(format!("  retain: {:?}", &self.preceding[next.node.pos]));
                    }
                    if self.preceding[next.node.pos].insert(current) {
                        self.open_set.push(next);
                    }
                    if is_target(&next) {
                        self.log(format!("  after:  {:?}", &self.preceding[next.node.pos]));
                    }
                }
            }
        }

        None
    }

    fn solve_path(&mut self, solution: day::ScoredNode) {
        let mut solution_path_nodes = HashSet::new();

        let mut node_gen: Vec<day::Node> = vec![solution.node];
        let mut next_node_gen: Vec<day::Node> = vec![];

        loop {
            if node_gen.is_empty() {
                break;
            }
            while let Some(node) = node_gen.pop() {
                let mut lowest_cost = u64::MAX;
                for prev in &self.preceding[node.pos] {
                    lowest_cost = lowest_cost.min(prev.f_score);
                }

                for prev in &self.preceding[node.pos] {
                    if prev.f_score == lowest_cost
                        && solution_path_nodes.insert(prev)
                        && prev.node.pos != self.maze.start_pos
                    {
                        next_node_gen.push(prev.node);
                    }
                }
            }
            std::mem::swap(&mut node_gen, &mut next_node_gen);
        }

        // let node_prev = &preceding[node.pos];
        //
        // let mut lowest_cost = u64::MAX;
        // for dir_set in node_prev {
        //     for node in dir_set {
        //         lowest_cost = lowest_cost.min(node.f_score);
        //     }
        // }

        // fn _solution_path(
        //     logs: &mut VecDeque<String>,
        //     path: &mut HashSet<day::Node>,
        //     preceding: &day::GridVec<[HashSet<day::ScoredNode>; 4]>,
        //     start_pos: day::Pos,
        //     mut node: day::Node,
        //     mut cost: usize,
        // ) {
        //     loop {
        //         if node.pos == start_pos && node.dir == day::Direction::default()
        //         /* || !path.insert(node) */
        //         {
        //             logs.push_back(format!("Solution path: {cost}"));
        //             break;
        //         }
        //
        //         path.insert(node);
        //
        //         let node_prev = &preceding[node];
        //
        //         if node_prev.is_empty() {
        //             unreachable!();
        //         } else if node_prev.len() == 1 {
        //             let p = *node_prev.iter().next().unwrap();
        //             if p.node.dir == node.dir {
        //                 cost += 1;
        //             } else {
        //                 cost += 1000;
        //             }
        //             node = p.node;
        //         } else {
        //             for &p in node_prev {
        //                 let next_cost = if p.node.dir == node.dir {
        //                     cost + 1
        //                 } else {
        //                     cost + 1000
        //                 };
        //                 _solution_path(logs, path, preceding, start_pos, p.node, next_cost);
        //             }
        //             break;
        //         }
        //     }
        // }
        //
        // _solution_path(
        //     &mut self.logs,
        //     &mut solution_path_nodes,
        //     &self.preceding,
        //     self.maze.start_pos,
        //     solution.node,
        //     0,
        // );

        self.solution_path
            .extend(solution_path_nodes.into_iter().map(|n| n.node.pos));

        self.log(format!(
            "Solution path has {} nodes",
            self.solution_path.len()
        ));
    }

    fn draw(&mut self, frame: &mut Frame) {
        // Layout

        let [mut maze_area, debug_area] =
            layout::Layout::horizontal([layout::Constraint::Fill(3), layout::Constraint::Fill(2)])
                .areas(frame.area());

        // Maze

        maze_area.width = maze_area.width.min(self.maze.width() as u16 + 2);
        maze_area.height = maze_area.height.min(self.maze.height() as u16 + 2);

        let rows = self
            .maze
            .rows
            .iter()
            .copied()
            .enumerate()
            .map(|(r, s)| self.str_to_maze_row(r, s));

        let cursor_cross_highlight_style = Style::new().white().on_light_blue();
        let cursor_highlight_style = Style::new().white().on_red();

        let table = widgets::Table::new(rows, vec![1; self.maze.width()])
            .column_spacing(0)
            .row_highlight_style(cursor_cross_highlight_style)
            .column_highlight_style(cursor_cross_highlight_style)
            .cell_highlight_style(cursor_highlight_style)
            .block(widgets::Block::bordered());
        frame.render_stateful_widget(table, maze_area, &mut self.maze_table_state);

        // Maze scrollbar

        let maze_scrollbar = widgets::Scrollbar::new(widgets::ScrollbarOrientation::VerticalRight)
            .symbols(symbols::scrollbar::VERTICAL)
            .begin_symbol(None)
            .end_symbol(None);
        frame.render_stateful_widget(
            maze_scrollbar,
            maze_area.inner(layout::Margin::new(0, 1)),
            &mut self.maze_scroll_state,
        );

        // Debug area

        let debug_block = widgets::Block::new()
            .borders(widgets::Borders::LEFT)
            .padding(widgets::Padding::new(1, 0, 1, 0));
        frame.render_widget(&debug_block, debug_area);

        let debug_inner_area = debug_block.inner(debug_area);

        // Debug info

        let cursor_pos: day::Pos = self.maze_table_state.selected_cell().unwrap().into();
        let mut debug_info_values = vec![
            ("Step".to_owned(), self.steps.to_string()),
            ("Open Set Count".to_owned(), self.open_set.len().to_string()),
            ("Cursor".to_owned(), cursor_pos.to_string()),
        ];
        let cursor_direction_preceding = &self.preceding[cursor_pos];
        let mut cursor_preceding = String::new();
        for sn in cursor_direction_preceding {
            write!(
                &mut cursor_preceding,
                "{} {} ({}) / ",
                sn.node.pos, sn.node.dir, sn.f_score
            )
            .unwrap();
        }
        debug_info_values.push(("  Preceding".into(), cursor_preceding));
        for dir in day::Direction::ALL_DIRECTIONS {
            let dist = self.dist[cursor_pos][dir as u8 as usize];
            debug_info_values.push((format!("       Dist {dir}"), format!("{dist}")))
        }

        let [debug_info_area, debug_lower_area] = layout::Layout::vertical([
            layout::Constraint::Length(debug_info_values.len() as u16),
            layout::Constraint::Fill(1),
        ])
        .spacing(1)
        .areas(debug_inner_area);

        let debug_info = debug_info_values
            .iter()
            .map(|(label, value)| text::Line::from_iter([label, ": ", value]))
            .collect::<text::Text>();
        frame.render_widget(widgets::Paragraph::new(debug_info), debug_info_area);

        // Logs

        let logs =
            widgets::Paragraph::new(self.logs.iter().map(String::as_str).collect::<text::Text>())
                .scroll((self.logs_scroll_offset as u16, 0))
                .block(widgets::Block::bordered());
        frame.render_widget(&logs, debug_lower_area);

        // Logs scrollbar

        let logs_scrollbar = widgets::Scrollbar::new(widgets::ScrollbarOrientation::VerticalRight)
            .symbols(symbols::scrollbar::VERTICAL)
            .begin_symbol(None)
            .end_symbol(None);
        self.logs_scroll_state = self.logs_scroll_state.content_length(self.logs.len());
        // self.logs_scroll_state = self.logs_scroll_state.content_length(self.logs.len());
        // let offset = self.logs_scroll_offset.min(4096);
        // self.logs_scroll_offset = offset;
        // self.logs_scroll_state = self.logs_scroll_state.position(offset);
        frame.render_stateful_widget(
            logs_scrollbar,
            debug_lower_area.inner(layout::Margin::new(0, 1)),
            &mut self.logs_scroll_state,
        );
    }

    fn str_to_maze_row(&self, r: usize, maze_row: &str) -> widgets::Row<'static> {
        widgets::Row::new(
            maze_row
                .bytes()
                .enumerate()
                .map(|(c, b)| self.byte_to_maze_cell(day::Pos::new(r, c), b)),
        )
    }

    fn byte_to_maze_cell(&self, pos: day::Pos, byte: u8) -> text::Span<'static> {
        match byte {
            b'#' => "█".gray(),
            b'.' => {
                if self.solution_path.contains(&pos) {
                    "@".green()
                } else {
                    " ".into()
                }
            }
            b'S' => "S".bold().yellow(),
            b'E' => "E".bold().yellow(),
            _ => unreachable!(),
        }
    }
}

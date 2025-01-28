use std::{
    cmp,
    collections::{BinaryHeap, HashMap},
    fmt,
    io::{self, Read, Write},
    ops,
};

use anyhow::Result;
use crossterm::{
    cursor, execute, queue,
    style::{self, Stylize},
    terminal,
};

crate::day_executors! {
    [part1]
    [part2]
}

crate::day_visualizers! {
    [part1_viz]
    []
}

const SEP: &str = " / ";

fn parse(input: &str) -> Maze {
    let mut rows = Vec::new();
    let mut start = Pos::default();
    let mut end = Pos::default();

    for (r, line) in input.lines().enumerate() {
        if let Some(c) = line.find('E') {
            end = Pos::new(r, c);
        }
        if let Some(c) = line.find('S') {
            start = Pos::new(r, c);
        }
        rows.push(line);
    }

    Maze {
        rows,
        start_pos: start,
        end_pos: end,
    }
}

struct Maze<'input> {
    rows: Vec<&'input str>,
    start_pos: Pos,
    end_pos: Pos,
}

impl Maze<'_> {
    /// https://en.wikipedia.org/wiki/A*_search_algorithm#Pseudocode
    ///
    /// ```
    /// // A* finds a path from start to goal.
    /// // h is the heuristic function. h(n) estimates the cost to reach goal from node n.
    /// function A_Star(start, goal, h)
    ///     // The set of discovered nodes that may need to be (re-)expanded.
    ///     // Initially, only the start node is known.
    ///     // This is usually implemented as a min-heap or priority queue rather than a hash-set.
    ///     openSet := {start}
    ///
    ///     // For node n, cameFrom[n] is the node immediately preceding it on the cheapest path from the start
    ///     // to n currently known.
    ///     cameFrom := an empty map
    ///
    ///     // For node n, gScore[n] is the currently known cost of the cheapest path from start to n.
    ///     gScore := map with default value of Infinity
    ///     gScore[start] := 0
    ///
    ///     // For node n, fScore[n] := gScore[n] + h(n). fScore[n] represents our current best guess as to
    ///     // how cheap a path could be from start to finish if it goes through n.
    ///     fScore := map with default value of Infinity
    ///     fScore[start] := h(start)
    ///
    ///     while openSet is not empty
    ///         // This operation can occur in O(Log(N)) time if openSet is a min-heap or a priority queue
    ///         current := the node in openSet having the lowest fScore[] value
    ///         if current = goal
    ///             return reconstruct_path(cameFrom, current)
    ///
    ///         openSet.Remove(current)
    ///         for each neighbor of current
    ///             // d(current,neighbor) is the weight of the edge from current to neighbor
    ///             // tentative_gScore is the distance from start to the neighbor through current
    ///             tentative_gScore := gScore[current] + d(current, neighbor)
    ///             if tentative_gScore < gScore[neighbor]
    ///                 // This path to neighbor is better than any previous one. Record it!
    ///                 cameFrom[neighbor] := current
    ///                 gScore[neighbor] := tentative_gScore
    ///                 fScore[neighbor] := tentative_gScore + h(neighbor)
    ///                 if neighbor not in openSet
    ///                     openSet.add(neighbor)
    ///
    ///     // Open set is empty but goal was never reached
    ///     return failure
    ///
    /// function reconstruct_path(cameFrom, current)
    ///     total_path := {current}
    ///     while current in cameFrom.Keys:
    ///         current := cameFrom[current]
    ///         total_path.prepend(current)
    ///     return total_path
    /// ```
    fn solve_astar(&self) -> Option<u64> {
        let start = Node::new(self.start_pos, Direction::default());

        // TODO: benchmark with a Fibonacci Heap
        // [crate](https://crates.io/crates/fibheap)
        let mut open_set = MinHeap::<ScoredNode>::new();
        open_set.push(ScoredNode::new(start));

        // TODO: try making this a `GridVec<[Node; 4]>`
        let mut preceding = HashMap::<Node, Node>::new();

        // TODO: try making this a `GridVec<[Node; 4]>`
        let mut g_scores = HashMap::<Node, u64>::new();
        g_scores.insert(start, 0);

        while let Some(current) = open_set.pop() {
            if current.node.pos == self.end_pos {
                open_set.clear();
                return Some(current.f_score);
            }

            let mut next = current;

            loop {
                let probe_pos = next.forward_pos();
                if self[probe_pos] == b'#' {
                    break;
                }
                next.node.pos = probe_pos;
                next.f_score += 1;
                if self[probe_pos.orthogonal1_to(current.node.dir)] != b'#'
                    || self[probe_pos.orthogonal2_to(current.node.dir)] != b'#'
                {
                    break;
                }
            }

            if next.node.pos != current.node.pos {
                let tentative_g_score = g_scores
                    .get(&current.node)
                    .map(|s| s + next.f_score - current.f_score)
                    .unwrap_or(u64::MAX);
                let next_g_score = *g_scores.get(&next.node).unwrap_or(&u64::MAX);
                if tentative_g_score < next_g_score {
                    preceding.insert(next.node, current.node);
                    g_scores.insert(next.node, tentative_g_score);
                    let f_score =
                        tentative_g_score + next.node.pos.manhattan_distance(self.end_pos);
                    if !open_set.contains(next.node) {
                        next.f_score = f_score;
                        open_set.push(next);
                    }
                }
            }

            let mut next = current.rotate_cw();
            let next_forward_pos = next.forward_pos();

            if self[next_forward_pos] != b'#' {
                let tentative_g_score = g_scores
                    .get(&current.node)
                    .map(|s| s + 1000)
                    .unwrap_or(u64::MAX);
                let next_g_score = *g_scores.get(&next.node).unwrap_or(&u64::MAX);
                if tentative_g_score < next_g_score {
                    preceding.insert(next.node, current.node);
                    g_scores.insert(next.node, tentative_g_score);
                    let f_score =
                        tentative_g_score + next.node.pos.manhattan_distance(self.end_pos);
                    if !open_set.contains(next.node) {
                        next.f_score = f_score;
                        open_set.push(next);
                    }
                }
            }

            let mut next = current.rotate_ccw();
            let next_forward_pos = next.forward_pos();

            if self[next_forward_pos] != b'#' {
                let tentative_g_score = g_scores
                    .get(&current.node)
                    .map(|s| s + 1000)
                    .unwrap_or(u64::MAX);
                let next_g_score = *g_scores.get(&next.node).unwrap_or(&u64::MAX);
                if tentative_g_score < next_g_score {
                    preceding.insert(next.node, current.node);
                    g_scores.insert(next.node, tentative_g_score);
                    let f_score =
                        tentative_g_score + next.node.pos.manhattan_distance(self.end_pos);
                    if !open_set.contains(next.node) {
                        next.f_score = f_score;
                        open_set.push(next);
                    }
                }
            }
        }

        None
    }

    fn viz_solve_astar(&self) {
        let mut stdout = io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).unwrap();
        terminal::enable_raw_mode().unwrap();

        let result = self._viz_solve_astar_impl(&mut stdout);

        execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show).unwrap();
        terminal::disable_raw_mode().unwrap();

        result.unwrap();
    }

    fn _viz_solve_astar_impl(&self, stdout: &mut io::Stdout) -> Result<Option<usize>> {
        // Algorithm setup

        let start = Node::new(self.start_pos, Direction::default());

        let mut open_set = MinHeap::<ScoredNode>::new();
        open_set.push(ScoredNode::new(start));

        let mut preceding = HashMap::<Node, Node>::new();

        let mut g_scores = HashMap::<Node, u64>::new();
        g_scores.insert(start, 0);

        // Viz loop

        let mut logs = Vec::<String>::new();
        let mut steps = 0_u32;
        for _ in 0..steps {
            self._viz_solve_astar_step(
                &mut logs,
                steps,
                &mut open_set,
                &mut preceding,
                &mut g_scores,
            );
        }

        self._viz_solve_astar_draw(
            stdout,
            &logs,
            steps,
            &preceding,
            open_set.0.as_slice(),
            None,
        )?;

        let mut inbuf = [0, 0, 0, 0];
        let mut n_read: usize;

        loop {
            n_read = io::stdin().read(&mut inbuf)?;

            match (n_read, inbuf[0]) {
                // Escape | ^C | ^D | q | Q
                (1, 0x1b | 0x03 | 0x04 | b'q' | b'Q') => break,
                // \r | Space
                (1, 0x0d | b' ') if !open_set.is_empty() => {
                    steps += 1;
                    let solution = self._viz_solve_astar_step(
                        &mut logs,
                        steps,
                        &mut open_set,
                        &mut preceding,
                        &mut g_scores,
                    );
                    self._viz_solve_astar_draw(
                        stdout,
                        &logs,
                        steps,
                        &preceding,
                        open_set.0.as_slice(),
                        solution,
                    )?;
                }
                _ => {}
            }
        }

        Ok(None)
    }

    fn _viz_solve_astar_step(
        &self,
        _logs: &mut Vec<String>,
        _step: u32,
        open_set: &mut MinHeap<ScoredNode>,
        preceding: &mut HashMap<Node, Node>,
        g_scores: &mut HashMap<Node, u64>,
    ) -> Option<Node> {
        if let Some(current) = open_set.pop() {
            if current.node.pos == self.end_pos {
                _logs.push(format!(
                    "{_step:4}| {} cost={}",
                    "SOLVED".bold().green(),
                    current.f_score.to_string().bold().green()
                ));
                open_set.clear();
                return Some(current.node);
            }

            let mut next = current;

            loop {
                let probe_pos = next.forward_pos();
                if self[probe_pos] == b'#' {
                    break;
                }
                next.node.pos = probe_pos;
                next.f_score += 1;
                if self[probe_pos.orthogonal1_to(current.node.dir)] != b'#'
                    || self[probe_pos.orthogonal2_to(current.node.dir)] != b'#'
                {
                    break;
                }
            }

            if next.node.pos != current.node.pos {
                let tentative_g_score = g_scores
                    .get(&current.node)
                    .map(|s| s + next.f_score - current.f_score)
                    .unwrap_or(u64::MAX);
                let next_g_score = *g_scores.get(&next.node).unwrap_or(&u64::MAX);
                if tentative_g_score < next_g_score {
                    preceding.insert(next.node, current.node);
                    g_scores.insert(next.node, tentative_g_score);
                    let f_score =
                        tentative_g_score + next.node.pos.manhattan_distance(self.end_pos);
                    if !open_set.contains(next.node) {
                        next.f_score = f_score;
                        open_set.push(next);
                    }
                }
            }

            let mut next = current.rotate_cw();
            let next_forward_pos = next.forward_pos();

            if self[next_forward_pos] != b'#' {
                let tentative_g_score = g_scores
                    .get(&current.node)
                    .map(|s| s + 1000)
                    .unwrap_or(u64::MAX);
                let next_g_score = *g_scores.get(&next.node).unwrap_or(&u64::MAX);
                if tentative_g_score < next_g_score {
                    preceding.insert(next.node, current.node);
                    g_scores.insert(next.node, tentative_g_score);
                    let f_score =
                        tentative_g_score + next.node.pos.manhattan_distance(self.end_pos);
                    if !open_set.contains(next.node) {
                        next.f_score = f_score;
                        open_set.push(next);
                    }
                }
            }

            let mut next = current.rotate_ccw();
            let next_forward_pos = next.forward_pos();

            if self[next_forward_pos] != b'#' {
                let tentative_g_score = g_scores
                    .get(&current.node)
                    .map(|s| s + 1000)
                    .unwrap_or(u64::MAX);
                let next_g_score = *g_scores.get(&next.node).unwrap_or(&u64::MAX);
                if tentative_g_score < next_g_score {
                    preceding.insert(next.node, current.node);
                    g_scores.insert(next.node, tentative_g_score);
                    let f_score =
                        tentative_g_score + next.node.pos.manhattan_distance(self.end_pos);
                    if !open_set.contains(next.node) {
                        next.f_score = f_score;
                        open_set.push(next);
                    }
                }
            }

            // fn g_scores_display(g_scores: &HashMap<Node, u64>) -> String {
            //     let mut out = String::new();
            //     for (i, (&node, &g)) in g_scores.iter().enumerate() {
            //         if i > 0 {
            //             out.push_str(SEP);
            //         }
            //         out.push_str(&format!("{},{} : {}", node.pos, node.dir, g));
            //     }
            //     out
            // }
            // _logs.push(format!("{_step:4}| open:   {open_set}"));
            // _logs.push(format!("{_step:4}| gScore: {}", g_scores_display(g_scores)));
        }

        None
    }

    fn _viz_solve_astar_draw(
        &self,
        stdout: &mut io::Stdout,
        logs: &[String],
        step: u32,
        preceding: &HashMap<Node, Node>,
        open_set: &[ScoredNode],
        solution: Option<Node>,
    ) -> Result<()> {
        let (term_width, term_height) = terminal::size()?;

        // Maze

        queue!(stdout, cursor::MoveTo(0, 0))?;
        for &row in &self.rows {
            queue!(stdout, terminal::Clear(terminal::ClearType::CurrentLine))?;
            for b in row.bytes() {
                let content = match b {
                    b'#' => "█".grey(),
                    b'.' => " ".stylize(),
                    b'S' => "S".bold().yellow(),
                    b'E' => "E".bold().yellow(),
                    _ => unreachable!(),
                };
                queue!(stdout, style::PrintStyledContent(content))?;
            }
            queue!(stdout, cursor::MoveToNextLine(1))?;
        }

        // Open Set

        queue!(stdout, cursor::SavePosition)?;

        for (i, ns) in open_set.iter().enumerate() {
            // if ns.node.pos == self.start_pos || ns.node.pos == self.end_pos {
            //     continue;
            // }
            queue!(
                stdout,
                cursor::MoveTo(ns.node.pos.col as u16, ns.node.pos.row as u16),
            )?;
            let mut content = match ns.node.dir {
                Direction::North => "^",
                Direction::East => ">",
                Direction::South => "v",
                Direction::West => "<",
            }
            .yellow();
            if i == 0 {
                content = content.on_yellow().black();
            }
            queue!(stdout, style::PrintStyledContent(content))?;
        }

        queue!(stdout, cursor::RestorePosition)?;

        // Solution Path

        if let Some(end) = solution {
            let path = "@".green();
            let path_cmd = style::PrintStyledContent(path);

            let mut next = end;

            while let Some(x) = preceding.get(&next).copied() {
                queue!(
                    stdout,
                    cursor::MoveTo(x.pos.col as u16, x.pos.row as u16),
                    path_cmd
                )?;
                next = x;
            }

            queue!(stdout, cursor::RestorePosition)?;
        }

        // Status

        queue!(
            stdout,
            cursor::MoveToNextLine(1),
            style::Print("Step: "),
            style::Print(step),
        )?;

        queue!(
            stdout,
            cursor::MoveToNextLine(1),
            style::Print("Next: "),
            terminal::Clear(terminal::ClearType::UntilNewLine),
        )?;
        if let Some(next) = open_set.first() {
            queue!(stdout, style::Print(next.node.pos))?;
        }

        // Logs

        queue!(
            stdout,
            cursor::MoveToNextLine(2),
            style::Print("━".repeat(term_width as usize)),
        )?;

        stdout.flush()?;

        queue!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown))?;

        let (_cur_col, cur_row) = cursor::position()?;

        let rows_remaining = term_height.saturating_sub(cur_row + 1) as usize;

        let log_chunks = logs
            .iter()
            .rev()
            .flat_map(|line| {
                let Some(i) = line.find(SEP) else {
                    return vec![line.as_str()].into_iter().rev();
                };

                let mut chunk_start = 0;
                let mut chunk_end = i;
                let mut chunks = vec![];

                while chunk_end < line.len() {
                    match line[chunk_end + SEP.len()..].find(SEP) {
                        Some(i) => {
                            if chunk_end - chunk_start + SEP.len() + i <= term_width as usize {
                                chunk_end += SEP.len() + i;
                            } else {
                                chunks.push(&line[chunk_start..chunk_end]);
                                chunk_start = chunk_end;
                                chunk_end += SEP.len() + i;
                            }
                        }
                        None => {
                            if line.len() - chunk_start <= term_width as usize {
                                chunks.push(&line[chunk_start..]);
                            } else {
                                chunks.push(&line[chunk_start..chunk_end]);
                                chunks.push(&line[chunk_end..]);
                            }
                            break;
                        }
                    }
                }

                chunks.into_iter().rev()
            })
            .take(rows_remaining)
            .collect::<Vec<_>>();

        for chunk in log_chunks.into_iter().rev() {
            queue!(stdout, cursor::MoveToNextLine(1), style::Print(chunk))?;
        }

        // for log in logs.iter().rev().take(rows_remaining).rev() {
        //     queue!(stdout, cursor::MoveToNextLine(1), style::Print(log))?;
        // }

        stdout.flush()?;

        Ok(())
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Pos {
    row: usize,
    col: usize,
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "r={:2} c={:2}", self.row, self.col)
        } else {
            write!(f, "r={},c={}", self.row, self.col)
        }
    }
}

impl Pos {
    fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    fn manhattan_distance(self, other: Self) -> u64 {
        let d_row = {
            let high = self.row.max(other.row) as u64;
            let low = self.row.min(other.row) as u64;
            high - low
        };
        let d_col = {
            let high = self.col.max(other.col) as u64;
            let low = self.col.min(other.col) as u64;
            high - low
        };
        d_row + d_col
    }

    #[inline(always)]
    fn nn(self) -> Self {
        Self {
            row: self.row - 1,
            col: self.col,
        }
    }

    #[inline(always)]
    fn ee(self) -> Self {
        Self {
            row: self.row,
            col: self.col + 1,
        }
    }

    #[inline(always)]
    fn ss(self) -> Self {
        Self {
            row: self.row + 1,
            col: self.col,
        }
    }

    #[inline(always)]
    fn ww(self) -> Self {
        Self {
            row: self.row,
            col: self.col - 1,
        }
    }

    #[inline(always)]
    fn move_in(self, dir: Direction) -> Self {
        match dir {
            Direction::North => self.nn(),
            Direction::East => self.ee(),
            Direction::South => self.ss(),
            Direction::West => self.ww(),
        }
    }

    #[inline(always)]
    fn orthogonal1_to(self, dir: Direction) -> Self {
        self.move_in(dir.rotate_cw())
    }

    #[inline(always)]
    fn orthogonal2_to(self, dir: Direction) -> Self {
        self.move_in(dir.rotate_ccw())
    }
}

impl ops::Index<Pos> for Maze<'_> {
    type Output = u8;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.rows[index.row].as_bytes()[index.col]
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
enum Direction {
    North,
    #[default]
    East,
    South,
    West,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::North => write!(f, "N"),
            Self::East => write!(f, "E"),
            Self::South => write!(f, "S"),
            Self::West => write!(f, "W"),
        }
    }
}

impl Direction {
    fn rotate_cw(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    fn rotate_ccw(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
            Self::West => Self::South,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
    pos: Pos,
    dir: Direction,
}

impl Node {
    fn new(pos: Pos, dir: Direction) -> Self {
        Self { pos, dir }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ScoredNode {
    node: Node,
    f_score: u64,
}

impl PartialOrd for ScoredNode {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredNode {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        other.f_score.cmp(&self.f_score)
    }
}

impl ScoredNode {
    fn new(node: Node) -> Self {
        Self { node, f_score: 0 }
    }

    fn forward_pos(self) -> Pos {
        self.node.pos.move_in(self.node.dir)
    }

    fn forward(mut self) -> Self {
        self.node.pos = self.node.pos.move_in(self.node.dir);
        self.f_score += 1;
        self
    }

    fn rotate_cw(mut self) -> Self {
        self.node.dir = self.node.dir.rotate_cw();
        self.f_score += 1000;
        self
    }

    fn rotate_ccw(mut self) -> Self {
        self.node.dir = self.node.dir.rotate_ccw();
        self.f_score += 1000;
        self
    }
}

struct MinHeap<T>(BinaryHeap<T>);

impl<T> ops::Deref for MinHeap<T> {
    type Target = BinaryHeap<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ops::DerefMut for MinHeap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for MinHeap<ScoredNode> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, sn) in self.0.as_slice().iter().enumerate() {
            if i > 0 {
                write!(f, "{SEP}")?;
            }
            write!(f, "{:5} {:#} {}", sn.f_score, sn.node.pos, sn.node.dir)?;
        }
        Ok(())
    }
}

impl<T: Ord> MinHeap<T> {
    fn new() -> Self {
        Self(BinaryHeap::new())
    }
}

impl MinHeap<ScoredNode> {
    fn contains(&self, node: Node) -> bool {
        self.0.as_slice().iter().any(|x| x.node == node)
    }
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let maze = parse(input);

    let cost = maze.solve_astar();

    cost.map(|c| Box::new(c) as Box<dyn std::fmt::Display>)
}

fn part1_viz(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let maze = parse(input);

    maze.viz_solve_astar();

    None
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    _ = input;

    None
}

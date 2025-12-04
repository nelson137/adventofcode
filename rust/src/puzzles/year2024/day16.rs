use std::{
    array, cmp,
    collections::{BinaryHeap, HashMap, HashSet},
    fmt,
    io::{self, Read, Write},
    mem, ops,
};

use anyhow::{Result, bail};
use crossterm::{
    cursor, queue,
    style::{self, Stylize},
    terminal,
};

inventory::submit!(
    crate::days::DayModule::new("2024", 16)
        .with_executors(
            crate::day_part_executors![part1],
            crate::day_part_executors![part2],
        )
        .with_pt1_visualizer(part1_viz)
        .with_pt2_visualizer(part2_viz)
);

mod viz;

fn parse(input: &str) -> Maze<'_> {
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

        let mut discovered = GridVec::<bool>::new(self.width(), self.height(), false);

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
                    if !discovered[next.node.pos] {
                        discovered[next.node.pos] = true;
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

    fn viz_solve_astar(&self) -> Option<usize> {
        viz::run(|stdout| self._viz_solve_astar_impl(stdout))
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
            n_read = match io::stdin().read(&mut inbuf) {
                Ok(n) => n,
                Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(err) => bail!(err),
            };

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
        let (term_width, _term_height) = terminal::size()?;

        // Maze

        viz::draw_maze(stdout, &self.rows)?;

        // Open Set

        queue!(stdout, cursor::SavePosition)?;

        viz::draw_open_set(stdout, open_set)?;

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

        viz::draw_logs(stdout, logs)?;

        Ok(())
    }

    fn width(&self) -> usize {
        self.rows[0].len()
    }

    fn height(&self) -> usize {
        self.rows.len()
    }

    /// https://en.wikipedia.org/wiki/A*_search_algorithm#Pseudocode
    ///
    ///
    /// ```
    /// function Dijkstra(Graph, source):
    ///     create vertex priority queue Q
    ///
    ///     dist[source] ← 0                          // Initialization
    ///     Q.add_with_priority(source, 0)            // associated priority equals dist[·]
    ///
    ///     for each vertex v in Graph.Vertices:
    ///         if v ≠ source
    ///             prev[v] ← UNDEFINED               // Predecessor of v
    ///             dist[v] ← INFINITY                // Unknown distance from source to v
    ///             // Q.add_with_priority(v, INFINITY) // XXX
    ///
    ///     while Q is not empty:                     // The main loop
    ///         u ← Q.extract_min()                   // Remove and return best vertex
    ///         for each neighbor v of u:             // Go through all v neighbors of u
    ///             alt ← dist[u] + Graph.Edges(u, v)
    ///             if alt < dist[v]:
    ///                 prev[v] ← u
    ///                 dist[v] ← alt
    ///                 // Q.decrease_priority(v, alt) // XXX
    ///                 Q.add_with_priority(v, alt)
    ///
    ///     return dist, prev
    /// ```
    fn solve_dijkstras(&self) -> Option<u64> {
        let start = Node::new(self.start_pos, Direction::default());

        let mut open_set = MinHeap::<ScoredNode>::new();
        open_set.push(ScoredNode::new(start));

        // for pos in self.rows.iter().copied().enumerate().flat_map(|(r, row)| {
        //     row.bytes()
        //         .enumerate()
        //         .filter_map(|(c, b)| (b == b'.').then_some(Pos::new(r, c)))
        // }) {
        //     if pos != self.start_pos {
        //         for dir in Direction::ALL_DIRECTIONS {
        //             open_set.push(ScoredNode::new(Node::new(pos, dir)));
        //         }
        //     }
        // }

        let mut dist = GridVec::new(self.width(), self.height(), [u64::MAX; 4]);

        let mut preceding = GridVec::new(self.width(), self.height(), array::from_fn(|_| vec![]));

        while let Some(current) = open_set.pop() {
            if current.node.pos == self.end_pos {
                return self._solve_dijkstras_count(&preceding);
            }

            {
                // let mut next = current;
                //
                // loop {
                //     let probe_pos = next.forward_pos();
                //     if self[probe_pos] == b'#' {
                //         break;
                //     }
                //     next.node.pos = probe_pos;
                //     next.f_score += 1;
                //     if self[probe_pos.orthogonal1_to(current.node.dir)] != b'#'
                //         || self[probe_pos.orthogonal2_to(current.node.dir)] != b'#'
                //     {
                //         break;
                //     }
                // }
                //
                // if next.node.pos != current.node.pos {
                //     let tentative_g_score = g_scores
                //         .get(&current.node)
                //         .map(|s| s + next.f_score - current.f_score)
                //         .unwrap_or(u64::MAX);
                //     let next_g_score = *g_scores.get(&next.node).unwrap_or(&u64::MAX);
                //     if tentative_g_score < next_g_score {
                //         preceding.insert(next.node, current.node);
                //         g_scores.insert(next.node, tentative_g_score);
                //         let f_score =
                //             tentative_g_score + next.node.pos.manhattan_distance(self.end_pos);
                //         if !open_set.contains(next.node) {
                //             next.f_score = f_score;
                //             open_set.push(next);
                //         }
                //     }
                // }

                let next = current.forward();

                if self[next.node.pos] != b'#' && next.f_score <= dist[next.node] {
                    preceding[next.node].push(current.node);
                    dist[next.node] = next.f_score;
                    open_set.push(next);
                }
            }

            {
                let next = current.rotate_cw();
                let next_forward_pos = next.forward_pos();

                if self[next_forward_pos] != b'#' && next.f_score <= dist[next.node] {
                    preceding[next.node].push(current.node);
                    dist[next.node] = next.f_score;
                    open_set.push(next);
                }
            }

            {
                let next = current.rotate_ccw();
                let next_forward_pos = next.forward_pos();

                if self[next_forward_pos] != b'#' && next.f_score <= dist[next.node] {
                    preceding[next.node].push(current.node);
                    dist[next.node] = next.f_score;
                    open_set.push(next);
                }
            }
        }

        None
    }

    fn _solve_dijkstras_count(&self, preceding: &GridVec<[Vec<Node>; 4]>) -> Option<u64> {
        let mut solution_path_nodes = HashSet::new();
        // solution_path_nodes.insert(end.node);

        // let mut node_gen = vec![end.node];
        let mut node_gen = vec![];
        let mut next_node_gen = vec![];

        for dir in Direction::ALL_DIRECTIONS {
            let node = Node::new(self.end_pos, dir);
            solution_path_nodes.insert(node);
            node_gen.push(node);
        }

        loop {
            if node_gen.is_empty() {
                break;
            }
            while let Some(node) = node_gen.pop() {
                for &prev in &*preceding[node] {
                    if solution_path_nodes.insert(prev) && prev.pos != self.start_pos {
                        next_node_gen.push(prev);
                    }
                }
            }
            mem::swap(&mut node_gen, &mut next_node_gen);
        }

        let solution_path_node_positions = solution_path_nodes
            .iter()
            .map(|n| n.pos)
            .collect::<HashSet<_>>();

        Some(solution_path_node_positions.len() as u64)
    }

    #[allow(dead_code, reason = "WIP")]
    fn viz_solve_dijkstras(&self) -> Option<u64> {
        viz::run(|stdout| self._viz_solve_dijkstras_impl(stdout))
    }

    fn _viz_solve_dijkstras_impl(&self, stdout: &mut io::Stdout) -> Result<Option<u64>> {
        // Algorithm setup

        let start = Node::new(self.start_pos, Direction::default());

        let mut open_set = MinHeap::<ScoredNode>::new();
        open_set.push(ScoredNode::new(start));

        let mut dist = GridVec::new(self.width(), self.height(), [u64::MAX; 4]);

        let mut preceding = GridVec::new(self.width(), self.height(), array::from_fn(|_| vec![]));

        // Viz loop

        let mut solution = None;

        let mut logs = Vec::<String>::new();
        let mut steps = 932393; // XXX: real input: end pos first discovered
        for _ in 0..steps {
            let current_solution = self._viz_solve_dijkstras_step(
                &mut logs,
                steps,
                &mut open_set,
                &mut dist,
                &mut preceding,
            );
            solution = solution.or(current_solution);
        }

        self._viz_solve_dijkstras_draw(
            stdout,
            &mut logs,
            steps,
            &preceding,
            open_set.0.as_slice(),
            solution,
        )?;

        let mut inbuf = [0, 0, 0, 0];
        let mut n_read: usize;

        loop {
            n_read = match io::stdin().read(&mut inbuf) {
                Ok(n) => n,
                Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(err) => bail!(err),
            };

            match (n_read, inbuf[0]) {
                // Escape | ^C | ^D | q | Q
                (1, 0x1b | 0x03 | 0x04 | b'q' | b'Q') => break,
                // \r | Space
                (1, 0x0d | b' ') if !open_set.is_empty() => {
                    steps += 1;
                    let current_solution = self._viz_solve_dijkstras_step(
                        &mut logs,
                        steps,
                        &mut open_set,
                        &mut dist,
                        &mut preceding,
                    );
                    solution = solution.or(current_solution);
                    self._viz_solve_dijkstras_draw(
                        stdout,
                        &mut logs,
                        steps,
                        &preceding,
                        open_set.0.as_slice(),
                        solution,
                    )?;
                }
                _ => {}
            }
        }

        Ok(solution.map(|n| n.f_score))
    }

    fn _viz_solve_dijkstras_step(
        &self,
        _logs: &mut Vec<String>,
        _step: u32,
        open_set: &mut MinHeap<ScoredNode>,
        dist: &mut GridVec<[u64; 4]>,
        preceding: &mut GridVec<[Vec<Node>; 4]>,
    ) -> Option<ScoredNode> {
        if let Some(current) = open_set.pop() {
            if current.node.pos == self.end_pos {
                return Some(current);
            }

            {
                let next = current.forward();
                let next_dist = dist[next.node];

                if self[next.node.pos] != b'#' && next.f_score <= next_dist {
                    preceding[next.node].push(current.node);
                    _logs.push(format!(
                        "{} {} -> {}",
                        current.node.pos, current.node.dir, next.node.pos
                    ));
                    dist[next.node] = next.f_score;
                    open_set.push(next);
                }
            }

            {
                let next = current.rotate_cw();
                let next_forward_pos = next.forward_pos();
                let next_dist = dist[next.node];

                if self[next_forward_pos] != b'#' && next.f_score <= next_dist {
                    preceding[next.node].push(current.node);
                    _logs.push(format!(
                        "{} {} -> {}",
                        current.node.pos, current.node.dir, next.node.dir
                    ));
                    dist[next.node] = next.f_score;
                    open_set.push(next);
                }
            }

            {
                let next = current.rotate_ccw();
                let next_forward_pos = next.forward_pos();
                let next_dist = dist[next.node];

                if self[next_forward_pos] != b'#' && next.f_score <= next_dist {
                    preceding[next.node].push(current.node);
                    _logs.push(format!(
                        "{} {} -> {}",
                        current.node.pos, current.node.dir, next.node.dir
                    ));
                    dist[next.node] = next.f_score;
                    open_set.push(next);
                }
            }
        }

        None
    }

    fn _viz_solve_dijkstras_draw(
        &self,
        stdout: &mut io::Stdout,
        logs: &mut Vec<String>,
        step: u32,
        preceding: &GridVec<[Vec<Node>; 4]>,
        open_set: &[ScoredNode],
        solution: Option<ScoredNode>,
    ) -> Result<()> {
        let (term_width, _term_height) = terminal::size()?;

        // Maze

        viz::draw_maze(stdout, &self.rows)?;

        // Open Set

        queue!(stdout, cursor::SavePosition)?;

        viz::draw_open_set(stdout, open_set)?;

        queue!(stdout, cursor::RestorePosition)?;

        // Solution Path

        if let Some(end) = solution {
            let path = "@".green();
            let path_cmd = style::PrintStyledContent(path);

            let mut solution_path_nodes = HashSet::new();
            solution_path_nodes.insert(end.node);

            let mut node_gen = vec![end.node];
            let mut next_node_gen = vec![];

            loop {
                if node_gen.is_empty() {
                    break;
                }
                while let Some(node) = node_gen.pop() {
                    for &prev in &*preceding[node] {
                        if solution_path_nodes.insert(prev) && prev.pos != self.start_pos {
                            next_node_gen.push(prev);
                        }
                    }
                }
                mem::swap(&mut node_gen, &mut next_node_gen);
            }

            let solution_path_node_positions = solution_path_nodes
                .iter()
                .map(|n| n.pos)
                .collect::<HashSet<_>>();
            logs.push(format!(
                "Solution path has {} nodes",
                solution_path_node_positions.len()
            ));

            for node in solution_path_nodes {
                queue!(
                    stdout,
                    cursor::MoveTo(node.pos.col as u16, node.pos.row as u16),
                    path_cmd
                )?;
            }

            stdout.flush()?;

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

        viz::draw_logs(stdout, logs)?;

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
            write!(f, "r={} c={}", self.row, self.col)
        }
    }
}

impl From<Pos> for (usize, usize) {
    fn from(value: Pos) -> Self {
        (value.row, value.col)
    }
}

impl From<(usize, usize)> for Pos {
    fn from((row, col): (usize, usize)) -> Self {
        Pos::new(row, col)
    }
}

impl Pos {
    const fn new(row: usize, col: usize) -> Self {
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
    const ALL_DIRECTIONS: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

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

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.pos, self.dir)
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

impl fmt::Debug for ScoredNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {}", self.node, self.f_score)
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
                write!(f, "{}", viz::SEP)?;
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

// #[derive(Clone, Copy, PartialEq, Eq, Hash)]
// struct ScoredPos {
//     pos: Pos,
//     dist: u64,
// }
//
// impl PartialOrd for ScoredPos {
//     fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }
//
// impl Ord for ScoredPos {
//     fn cmp(&self, other: &Self) -> cmp::Ordering {
//         other.dist.cmp(&self.dist)
//     }
// }
//
// impl ScoredPos {
//     fn new(pos: Pos) -> Self {
//         Self {
//             pos,
//             dist: u64::MAX,
//         }
//     }
// }

struct GridVec<T> {
    width: usize,
    grid: Vec<T>,
}

impl<T: Clone> GridVec<T> {
    fn new(width: usize, height: usize, init: T) -> Self {
        Self {
            width,
            grid: vec![init; width * height],
        }
    }
}

impl<T> ops::Index<Pos> for GridVec<T> {
    type Output = T;

    fn index(&self, pos: Pos) -> &Self::Output {
        &self.grid[pos.row * self.width + pos.col]
    }
}

impl<T> ops::IndexMut<Pos> for GridVec<T> {
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        &mut self.grid[pos.row * self.width + pos.col]
    }
}

impl<T> ops::Index<Node> for GridVec<[T; 4]> {
    type Output = T;

    fn index(&self, node: Node) -> &Self::Output {
        &self[node.pos][node.dir as u8 as usize]
    }
}

impl<T> ops::IndexMut<Node> for GridVec<[T; 4]> {
    fn index_mut(&mut self, node: Node) -> &mut Self::Output {
        &mut self[node.pos][node.dir as u8 as usize]
    }
}

impl<T> ops::Index<ScoredNode> for GridVec<[T; 4]> {
    type Output = T;

    fn index(&self, sn: ScoredNode) -> &Self::Output {
        &self[sn.node]
    }
}

impl<T> ops::IndexMut<ScoredNode> for GridVec<[T; 4]> {
    fn index_mut(&mut self, sn: ScoredNode) -> &mut Self::Output {
        &mut self[sn.node]
    }
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let maze = parse(input);

    let cost = maze.solve_astar();

    cost.map(|c| Box::new(c) as Box<dyn std::fmt::Display>)
}

fn part1_viz(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let maze = parse(input);

    let cost = maze.viz_solve_astar();

    cost.map(|c| Box::new(c) as Box<dyn std::fmt::Display>)
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let maze = parse(input);

    let cost = maze.solve_dijkstras();

    cost.map(|c| Box::new(c) as Box<dyn std::fmt::Display>)
}

fn part2_viz(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let maze = parse(input);

    // let cost = maze.viz_solve_dijkstras();
    // cost.map(|c| Box::new(c) as Box<dyn std::fmt::Display>)

    viz::run_dijkstras(&maze);

    None
}

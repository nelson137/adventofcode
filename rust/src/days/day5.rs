use std::{cmp::Ordering, iter};

fn parse(input: &str) -> (PrintRules, Vec<Vec<u32>>) {
    let mut rules = Vec::<(u32, u32)>::new();
    let mut updates = Vec::<Vec<u32>>::new();

    let mut parsing_rules = true;

    for line in input.lines() {
        if line.is_empty() {
            parsing_rules = false;
            continue;
        }
        if parsing_rules {
            let (a, b) = line.split_once('|').unwrap();
            let a = a.parse::<u32>().unwrap();
            let b = b.parse::<u32>().unwrap();
            rules.push((a, b));
        } else {
            let update = line.split(',').map(|u| u.parse::<u32>().unwrap()).collect();
            updates.push(update);
        }
    }

    (PrintRules::new(rules), updates)
}

struct PrintRules {
    max_page: usize,
    rules_adjacent: Vec<bool>,
}

impl PrintRules {
    fn new(rules: Vec<(u32, u32)>) -> Self {
        let max_page = rules
            .iter()
            .copied()
            .flat_map(|(a, b)| iter::once(a).chain(iter::once(b)))
            .max()
            .unwrap();
        let max_page = max_page as usize + 1;

        let mut rules_adjacent = vec![false; max_page * max_page];

        for &(a, b) in &rules {
            rules_adjacent[a as usize * max_page + b as usize] = true;
        }

        Self {
            max_page,
            rules_adjacent,
        }
    }

    fn update_is_valid(&self, update: &[u32]) -> bool {
        for (i, u) in update.iter().copied().enumerate().rev() {
            for &prev in &update[..i] {
                if self.rules_adjacent[u as usize * self.max_page + prev as usize] {
                    return false;
                }
            }
        }
        true
    }

    fn adjacent(&self, a: u32, b: u32) -> bool {
        let a = a as usize;
        let b = b as usize;
        self.rules_adjacent[a * self.max_page + b]
    }
}

pub(super) fn part1(input: &str) -> Box<dyn std::fmt::Display> {
    let (print_rules, updates) = parse(input);

    let answer = updates
        .iter()
        .filter(|update| print_rules.update_is_valid(update))
        .map(|update| update[update.len() / 2])
        .sum::<u32>();

    Box::new(answer)
}

pub(super) fn part2(input: &str) -> Box<dyn std::fmt::Display> {
    let (print_rules, mut updates) = parse(input);

    let answer = updates
        .iter_mut()
        .filter(|update| !print_rules.update_is_valid(update))
        .map(|update| {
            update.sort_by(|&a, &b| {
                if print_rules.adjacent(a, b) {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            });
            update[update.len() / 2]
        })
        .sum::<u32>();

    Box::new(answer)
}

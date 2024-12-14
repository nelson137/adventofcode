use std::{cmp::Ordering, iter};

struct PrintUpdateOrganizer {
    max_page: usize,
    rules_adjacent: Vec<bool>,
    updates: Vec<Vec<u32>>,
}

impl PrintUpdateOrganizer {
    fn new(input: &str) -> Self {
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
            updates,
        }
    }
}

fn update_is_valid(update: &[u32], rules_adjacent: &[bool], max_page: usize) -> bool {
    for (i, u) in update.iter().copied().enumerate().rev() {
        for &prev in &update[..i] {
            if rules_adjacent[u as usize * max_page + prev as usize] {
                return false;
            }
        }
    }
    true
}

pub(super) fn part1(input: &str) -> Box<dyn std::fmt::Display> {
    let print_organizer = PrintUpdateOrganizer::new(input);

    let answer = print_organizer
        .updates
        .iter()
        .filter(|update| {
            update_is_valid(
                update,
                &print_organizer.rules_adjacent,
                print_organizer.max_page,
            )
        })
        .map(|update| update[update.len() / 2])
        .sum::<u32>();

    Box::new(answer)
}

pub(super) fn part2(input: &str) -> Box<dyn std::fmt::Display> {
    let mut print_organizer = PrintUpdateOrganizer::new(input);

    let answer = print_organizer
        .updates
        .iter_mut()
        .filter(|update| {
            !update_is_valid(
                update,
                &print_organizer.rules_adjacent,
                print_organizer.max_page,
            )
        })
        .map(|update| {
            update.sort_by(|&a, &b| {
                if print_organizer.rules_adjacent
                    [a as usize * print_organizer.max_page + b as usize]
                {
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

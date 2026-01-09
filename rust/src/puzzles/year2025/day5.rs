use std::{cmp, ops};

inventory::submit!(crate::days::DayModule::new(2025, 5).with_executors(
    crate::day_part_executors![part1],
    crate::day_part_executors![part2],
));

struct IngredientDatabase<It> {
    fresh_ingredient_id_range_buckets: Vec<IdRangeBucket>,
    available_ingredient_ids_iter: It,
}

struct IdRangeBucket {
    ranges: Vec<ops::RangeInclusive<u64>>,
    max: u64,
}

impl IdRangeBucket {
    fn new(range: ops::RangeInclusive<u64>) -> Self {
        let max = *range.end();
        Self {
            ranges: vec![range],
            max,
        }
    }
}

fn parse_ingredient_database(input: &str) -> IngredientDatabase<impl Iterator<Item = &str>> {
    let mut iter = input.lines();
    let mut fresh_ingredient_id_ranges = Vec::new();

    loop {
        let Some(line) = iter.next() else {
            break;
        };
        if line.is_empty() {
            break;
        }

        let (min, max) = line.split_once('-').unwrap();
        let range = min.parse::<u64>().unwrap()..=max.parse().unwrap();
        fresh_ingredient_id_ranges.push(range);
    }

    // Sort by range start asc.
    fresh_ingredient_id_ranges.sort_unstable_by(|a, b| a.start().cmp(b.start()));

    let mut fresh_ingredient_id_range_buckets = Vec::<IdRangeBucket>::new();

    'ranges: for range in &fresh_ingredient_id_ranges {
        for bucket in &mut fresh_ingredient_id_range_buckets {
            if *range.start() > bucket.max {
                bucket.ranges.push(range.clone());
                bucket.max = cmp::max(bucket.max, *range.end());
                continue 'ranges;
            }
        }
        fresh_ingredient_id_range_buckets.push(IdRangeBucket::new(range.clone()));
    }

    IngredientDatabase {
        fresh_ingredient_id_range_buckets,
        available_ingredient_ids_iter: iter,
    }
}

impl<'input, It> IngredientDatabase<It>
where
    It: Iterator<Item = &'input str>,
{
    fn count_fresh_ids(&mut self) -> usize {
        let mut count = 0_usize;

        for line in &mut self.available_ingredient_ids_iter {
            let id: u64 = line.parse().unwrap();

            if self.fresh_ingredient_id_range_buckets.iter().any(move |b| {
                b.ranges
                    .binary_search_by(move |r| match () {
                        _ if r.contains(&id) => cmp::Ordering::Equal,
                        _ if *r.end() < id => cmp::Ordering::Less,
                        _ => cmp::Ordering::Greater,
                    })
                    .is_ok()
            }) {
                count += 1;
            }
        }

        count
    }
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut db = parse_ingredient_database(input);

    let fresh_ingredient_id_count = db.count_fresh_ids();

    Some(Box::new(fresh_ingredient_id_count))
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut fresh_ingredient_id_ranges = Vec::new();

    for line in input.lines() {
        if line.is_empty() {
            break;
        }

        let (min, max) = line.split_once('-').unwrap();
        let start = min.parse::<u64>().unwrap();
        let end = max.parse().unwrap();
        fresh_ingredient_id_ranges.push(start..=end);
    }

    loop {
        let mut has_overlapping = false;

        for i in (0..fresh_ingredient_id_ranges.len()).rev() {
            let r = &fresh_ingredient_id_ranges[i];
            let (r_start, r_end) = (*r.start(), *r.end());
            if r_start == u64::MAX {
                continue;
            }

            for j in 0..fresh_ingredient_id_ranges.len() {
                if j == i {
                    continue;
                }
                let range = &mut fresh_ingredient_id_ranges[j];
                if *range.start() == u64::MAX {
                    continue;
                }
                if range.contains(&r_start) || range.contains(&r_end) {
                    has_overlapping = true;
                    let start = cmp::min(*range.start(), r_start);
                    let end = cmp::max(*range.end(), r_end);
                    *range = start..=end;
                    fresh_ingredient_id_ranges[i] = u64::MAX..=u64::MAX;
                    break;
                }
            }
        }

        if !has_overlapping {
            break;
        }
    }

    let count = fresh_ingredient_id_ranges
        .iter()
        .filter(|r| *r.start() != u64::MAX)
        .map(|r| *r.end() - *r.start() + 1)
        .sum::<u64>();

    Some(Box::new(count))
}

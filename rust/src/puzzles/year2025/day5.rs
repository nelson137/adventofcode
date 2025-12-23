use std::ops;

inventory::submit!(crate::days::DayModule::new(2025, 5).with_executors(
    crate::day_part_executors![part1],
    crate::day_part_executors![part2],
));

struct IngredientDatabase<It> {
    fresh_ingredient_id_ranges: Vec<ops::RangeInclusive<u64>>,
    available_ingredient_ids_iter: It,
}

fn parse_ingredient_database(input: &str) -> IngredientDatabase<impl Iterator<Item = &str>> {
    let mut iter = input.lines();
    let mut fresh_ingredient_id_ranges = Vec::<ops::RangeInclusive<u64>>::new();

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

    IngredientDatabase {
        fresh_ingredient_id_ranges,
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
            for range in &self.fresh_ingredient_id_ranges {
                if range.contains(&id) {
                    count += 1;
                    break;
                }
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
    _ = input;

    None
}

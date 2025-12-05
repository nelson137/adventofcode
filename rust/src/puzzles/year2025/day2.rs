use std::fmt::Write;

use adventofcode::count_digits;

inventory::submit!(crate::days::DayModule::new("2025", 2).with_executors(
    crate::day_part_executors![part1_brute],
    crate::day_part_executors![part2],
));

#[derive(Debug)]
struct ProductIdRange<'input> {
    raw_start: &'input str,
    raw_end: &'input str,
}

fn parse_product_id_ranges(input: &str) -> impl Iterator<Item = ProductIdRange<'_>> {
    input.trim().split(',').map(|raw_range| {
        let (raw_start, raw_end) = raw_range.split_once('-').unwrap();
        ProductIdRange { raw_start, raw_end }
    })
}

fn part1_brute(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut id_string = String::new();
    let mut invalid_id_acc = 0_u64;

    for ProductIdRange { raw_start, raw_end } in parse_product_id_ranges(input) {
        let start_len = raw_start.len();
        let end_len = raw_end.len();

        let is_start_len_even = start_len % 2 == 0;
        let is_end_len_even = end_len % 2 == 0;

        if !is_start_len_even && !is_end_len_even {
            continue;
        }

        let start: u64 = raw_start.parse().unwrap();
        let end: u64 = raw_end.parse().unwrap();

        for id in start..=end {
            id_string.clear();
            write!(&mut id_string, "{id}").unwrap();

            let (l, r) = id_string.split_at(id_string.len() / 2);
            if l == r {
                invalid_id_acc += id;
            }
        }
    }

    Some(Box::new(invalid_id_acc))
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    _ = input;

    None
}

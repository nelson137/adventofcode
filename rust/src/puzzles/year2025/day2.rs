use std::fmt::Write;

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

    for id_range in parse_product_id_ranges(input) {
        let start: u64 = id_range.raw_start.parse().unwrap();
        let end: u64 = id_range.raw_end.parse().unwrap();

        for id in start..=end {
            id_string.clear();
            write!(&mut id_string, "{id}").unwrap();
            if id_string.len() % 2 == 0
                && let (l, r) = id_string.split_at(id_string.len() / 2)
                && l == r
            {
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

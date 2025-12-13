use std::fmt::Write;

use adventofcode::count_digits;

inventory::submit!(crate::days::DayModule::new("2025", 2).with_executors(
    crate::day_part_executors![part1_fast, part1_brute],
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

fn repeat_id(half_id: u64) -> u64 {
    let n_digits = if half_id == 0 {
        1
    } else {
        count_digits(half_id)
    };
    half_id * 10_u64.pow(n_digits as u32) + half_id
}

fn part1_fast(input: &str) -> Option<Box<dyn std::fmt::Display>> {
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

        let half_start: u64 = if is_start_len_even {
            raw_start.split_at(start_len / 2).0.parse().unwrap()
        } else {
            10_u64.pow((start_len / 2) as u32)
        };

        let half_end: u64 = if is_end_len_even {
            raw_end.split_at(end_len / 2).0.parse().unwrap()
        } else {
            10_u64.pow((end_len / 2) as u32) - 1
        };

        {
            let mut half_id = half_start - 1;
            let mut id = repeat_id(half_id);
            while id >= start {
                invalid_id_acc += id;
                half_id -= 1;
                id = repeat_id(half_id);
            }
        }

        invalid_id_acc += (half_start..=half_end)
            .map(repeat_id)
            .filter(|&id| start <= id && id <= end)
            .sum::<u64>();

        {
            let mut half_id = half_end + 1;
            let mut id = repeat_id(half_id);
            while id <= end {
                invalid_id_acc += id;
                half_id += 1;
                id = repeat_id(half_id);
            }
        }
    }

    Some(Box::new(invalid_id_acc))
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

fn is_repeated_digits_of_len(chunk_len: usize, id: &str) -> bool {
    if id.len() % chunk_len != 0 {
        return false;
    }

    let mut chunk1 = &id[..chunk_len];
    let mut i = chunk_len;

    while i < id.len() {
        let next_i = i + chunk_len;
        let chunk2 = &id[i..next_i];
        if chunk1 != chunk2 {
            return false;
        }
        chunk1 = chunk2;
        i = next_i;
    }

    true
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut id_string = String::new();
    let mut invalid_id_acc = 0_u64;

    for ProductIdRange { raw_start, raw_end } in parse_product_id_ranges(input) {
        let start: u64 = raw_start.parse().unwrap();
        let end: u64 = raw_end.parse().unwrap();

        for id in start..=end {
            id_string.clear();
            write!(&mut id_string, "{id}").unwrap();

            for len in 1..=id_string.len() / 2 {
                if is_repeated_digits_of_len(len, &id_string) {
                    invalid_id_acc += id;
                    break;
                }
            }
        }
    }

    Some(Box::new(invalid_id_acc))
}

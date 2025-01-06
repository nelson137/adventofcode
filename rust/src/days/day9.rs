use std::collections::VecDeque;

crate::day_executors! {
    [part1]
    [part2]
}

crate::day_visualizers! {
    []
    []
}

#[inline(always)]
fn parse_byte(b: u8) -> u32 {
    match b {
        b'0'..=b'9' => (b - b'0') as u32,
        _ => unsafe { std::hint::unreachable_unchecked() },
    }
}

pub(super) fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let diskmap_str = input.trim();

    let disk_size = diskmap_str.bytes().map(parse_byte).sum::<u32>() as usize;
    let mut file_block_map = Vec::<bool>::with_capacity(disk_size);
    let mut file_blocks = VecDeque::<u32>::with_capacity(disk_size);

    let mut block_is_file = true;
    let mut file_id = 0;
    for b in diskmap_str.bytes() {
        for _ in 0..parse_byte(b) {
            if block_is_file {
                file_block_map.push(true);
                file_blocks.push_back(file_id);
            } else {
                file_block_map.push(false);
            }
        }

        if block_is_file {
            file_id += 1;
        }
        block_is_file = !block_is_file;
    }

    let mut checksum = 0_usize;
    for (i, block_is_file) in file_block_map.iter().copied().enumerate() {
        let Some(fblock_id) = (if block_is_file {
            file_blocks.pop_front()
        } else {
            file_blocks.pop_back()
        }) else {
            break;
        };

        checksum += i * fblock_id as usize;
    }

    Some(Box::new(checksum))
}

pub(super) fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    _ = input;

    None
}

use std::collections::VecDeque;

crate::day_executors! {
    [part1_v2, part1]
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

/// A span checksum is a subset of the full checksum spanning `blen` blocks
/// belonging to the same file (`fid`) starting at block index `start_bid`.
///
/// It can be calculated with the following code:
///
/// ```
/// let mut checksum = 0;
/// for i in 0..(blen as usize) {
///     checksum += (start_bid + i) * fid;
/// }
/// ```
///
/// which is represented by the following equation:
///
/// ```
/// cksum = Σ ( fid * (start_bid + i) )  { i | 0 <= i < blen }
/// ```
///
/// Using simple algebra we can rearrange this equation to remove the summation
/// for-loop:
///
/// ```
/// cksum = (fid * (start_bid + 0)) + (fid * (start_bid + 1)) + (fid * (start_bid + 2)) + ..
///       = fid * ((start_bid + 0) + (start_bid + 1) + (start_bid + 2) + ..)
///       = fid * ((blen * start_bid) + 0 + 1 + 2 + ..)
///       = fid * ((blen * start_bid) + Σ_0^blen)
///       = fid * ((blen * start_bid) + (blen * (blen - 1) / 2))
/// ```
///
/// Note the last two lines. The expression `0 + 1 + 2 + ..` is the sum of
/// numbers from `0` to `n` where `n=blen`, for which there is the constant-time
/// equation `n * (n - 1) / 2`.
fn span_checksum(start_bid: usize, blen: u32, fid: usize) -> usize {
    let blen = blen as usize;
    fid * (blen * start_bid + (blen * (blen - 1) / 2))
}

pub(super) fn part1_v2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let diskmap_str = input.trim();
    let mut diskmap = Vec::<u32>::with_capacity(diskmap_str.len());

    let mut n_space_blocks = 0_usize;
    let mut span_is_space = false;
    for b in diskmap_str.bytes() {
        let span = parse_byte(b);
        diskmap.push(span);
        if span_is_space {
            n_space_blocks += span as usize;
        }
        span_is_space = !span_is_space;
    }

    let mut end_forward_walk = diskmap.len() - 1;
    while n_space_blocks > 0 {
        n_space_blocks = n_space_blocks.saturating_sub(diskmap[end_forward_walk] as usize);
        end_forward_walk -= 1;
    }

    let mut bid = 0_usize;
    let mut span_is_file = true;
    let mut l_fid = 0;
    let mut r_fid = (diskmap.len() - 1) / 2;
    let mut r_blen = diskmap[r_fid * 2];
    let mut checksum = 0_usize;

    for len in diskmap[..end_forward_walk].iter().copied() {
        if span_is_file {
            checksum += span_checksum(bid, len, l_fid);
            l_fid += 1;
            span_is_file = !span_is_file;
            bid += len as usize;
            continue;
        }

        let mut span_len = len;

        while span_len > 0 {
            let min_len = span_len.min(r_blen);
            span_len -= min_len;
            r_blen -= min_len;
            checksum += span_checksum(bid, min_len, r_fid);
            bid += min_len as usize;

            if r_blen == 0 {
                r_fid -= 1;
                r_blen = diskmap[r_fid * 2];
            }
        }

        span_is_file = !span_is_file;
    }

    checksum += span_checksum(bid, r_blen, r_fid);

    Some(Box::new(checksum))
}

pub(super) fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    _ = input;

    None
}

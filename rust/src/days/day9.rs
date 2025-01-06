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
fn parse_byte32(b: u8) -> u32 {
    match b {
        b'0'..=b'9' => (b - b'0') as u32,
        _ => unsafe { std::hint::unreachable_unchecked() },
    }
}

#[inline(always)]
fn parse_byte8(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        _ => unsafe { std::hint::unreachable_unchecked() },
    }
}

pub(super) fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let diskmap_str = input.trim();

    let disk_size = diskmap_str.bytes().map(parse_byte32).sum::<u32>() as usize;
    let mut file_block_map = Vec::<bool>::with_capacity(disk_size);
    let mut file_blocks = VecDeque::<u32>::with_capacity(disk_size);

    let mut block_is_file = true;
    let mut file_id = 0;
    for b in diskmap_str.bytes() {
        for _ in 0..parse_byte32(b) {
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
fn span_checksum(start_bid: u32, blen: u32, fid: usize) -> usize {
    let blen = blen as usize;
    fid * (blen * start_bid as usize + (blen * (blen - 1) / 2))
}

pub(super) fn part1_v2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let diskmap_str = input.trim();
    let mut diskmap = Vec::<u32>::with_capacity(diskmap_str.len());

    let mut n_file_blocks = 0_usize;
    for (i, b) in diskmap_str.bytes().enumerate() {
        let span = parse_byte32(b);
        diskmap.push(span);
        if i % 2 == 0 {
            n_file_blocks += span as usize;
        }
    }

    let mut n_checked_file_blocks = 0_usize;
    let mut bid = 0_u32;
    let mut span_is_file = true;
    let mut l_fid = 0;
    let mut r_fid = (diskmap.len() - 1) / 2;
    let mut r_blen = diskmap[r_fid * 2];
    let mut checksum = 0_usize;

    for len in diskmap.iter().copied() {
        if span_is_file {
            n_checked_file_blocks += len as usize;
            if n_checked_file_blocks >= n_file_blocks {
                break;
            }

            checksum += span_checksum(bid, len, l_fid);
            l_fid += 1;
            span_is_file = !span_is_file;
            bid += len;
            continue;
        }

        let mut span_len = len;

        while span_len > 0 {
            let min_len = span_len.min(r_blen);
            n_checked_file_blocks += min_len as usize;

            span_len -= min_len;
            r_blen -= min_len;
            checksum += span_checksum(bid, min_len, r_fid);
            bid += min_len;

            if r_blen == 0 {
                r_fid -= 1;
                r_blen = diskmap[r_fid * 2];
            }
        }

        if n_checked_file_blocks >= n_file_blocks {
            break;
        }

        span_is_file = !span_is_file;
    }

    checksum += span_checksum(bid, r_blen, r_fid);

    Some(Box::new(checksum))
}

#[derive(Clone, Copy, Debug)]
struct FileSpan {
    start_bid: u32,
    len: u8,
}

#[derive(Clone, Copy, Debug)]
struct SpaceSpan {
    start_bid: u32,
    original_len: u8,
    len: u8,
}

impl SpaceSpan {
    fn space_offset(self) -> u8 {
        self.original_len - self.len
    }
}

pub(super) fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let diskmap_str = input.trim();

    let mut file_spans = Vec::<FileSpan>::with_capacity(diskmap_str.len() / 2 + 1);
    let mut space_spans = Vec::<SpaceSpan>::with_capacity(diskmap_str.len() / 2 + 1);

    let mut bid = 0;
    for (i, b) in diskmap_str.bytes().enumerate() {
        let len = parse_byte8(b);
        if i % 2 == 0 {
            file_spans.push(FileSpan {
                start_bid: bid,
                len,
            });
        } else {
            space_spans.push(SpaceSpan {
                start_bid: bid,
                original_len: len,
                len,
            });
        }
        bid += len as u32;
    }

    let mut r_fi = file_spans.len() - 1;
    let mut checksum = 0_usize;

    while r_fi > 0 {
        let file = file_spans[r_fi];

        let mut l_si = 0_usize;
        let mut l_bid = file_spans[0].len as u32;
        let did_move_file = loop {
            let space = &mut space_spans[l_si];

            if space.start_bid >= file.start_bid {
                break false;
            }

            if file.len <= space.len {
                let space_offset = space.space_offset();
                space.len -= file.len;
                checksum += span_checksum(l_bid + space_offset as u32, file.len as u32, r_fi);
                break true;
            }

            l_si += 1;
            l_bid += space.original_len as u32 + file_spans[l_si].len as u32;
        };

        if !did_move_file {
            checksum += span_checksum(file.start_bid, file.len as u32, r_fi);
        }

        r_fi -= 1;
    }

    Some(Box::new(checksum))
}

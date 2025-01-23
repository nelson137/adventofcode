use std::{collections, hash};

#[inline(always)]
pub fn count_digits(x: u64) -> u64 {
    _count_digits_fast(x)
}

#[cfg(any(test, feature = "bench"))]
#[inline(always)]
pub fn _count_digits_with_log(i: u64) -> u64 {
    (i as f64 + 0.1).log10().ceil() as u64
}

#[inline(always)]
pub fn _count_digits_fast(v: u64) -> u64 {
    log10(v) + 1
}

/// [Source](https://graphics.stanford.edu/~seander/bithacks.html#IntegerLog10)
#[inline(always)]
fn log10(v: u64) -> u64 {
    #[rustfmt::skip]
    static POWERS_OF_TEN: [u64; 20] = [
        1,                         10,                         100,
        1_000,                     10_000,                     100_000,
        1_000_000,                 10_000_000,                 100_000_000,
        1_000_000_000,             10_000_000_000,             100_000_000_000,
        1_000_000_000_000,         10_000_000_000_000,         100_000_000_000_000,
        1_000_000_000_000_000,     10_000_000_000_000_000,     100_000_000_000_000_000,
        1_000_000_000_000_000_000, 10_000_000_000_000_000_000,
    ];

    let t = ((log2(v) + 1) * 1233) >> 12;
    t - (if v < POWERS_OF_TEN[t as usize] { 1 } else { 0 })
}

/// [Source](https://graphics.stanford.edu/~seander/bithacks.html#IntegerLogDeBruijn)
/// [Source](https://stackoverflow.com/a/36026194)
#[inline(always)]
pub fn log2(mut v: u64) -> u64 {
    static MULTIPLY_DEBRUIJN_BIT_POS: [u64; 64] = [
        0, 47, 1, 56, 48, 27, 2, 60, 57, 49, 41, 37, 28, 16, 3, 61, 54, 58, 35, 52, 50, 42, 21, 44,
        38, 32, 29, 23, 17, 11, 4, 62, 46, 55, 26, 59, 40, 36, 15, 53, 34, 51, 20, 43, 31, 22, 10,
        45, 25, 39, 14, 33, 19, 30, 9, 24, 13, 18, 8, 12, 7, 6, 5, 63,
    ];

    // round up to one less than the next highest power of 2
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v |= v >> 32;

    MULTIPLY_DEBRUIJN_BIT_POS[0x03F79D71B4CB0A89_u64.wrapping_mul(v) as usize >> 58]
}

pub type HashMap<K, V> = collections::HashMap<K, V, hash::BuildHasherDefault<Murmur3MixHash64>>;

#[derive(Default)]
pub struct Murmur3MixHash64 {
    value: u64,
}

impl hash::Hasher for Murmur3MixHash64 {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.value
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        debug_assert_eq!(bytes.len(), 8);
        let ptr = bytes.as_ptr() as *const [u8; 8];
        // SAFETY: we know bytes will have length 8 because it will always be a u64
        self.value = u64::from_ne_bytes(unsafe { *ptr });

        // The 64-bit finalizer mixer from MurmerHash3:
        // https://github.com/aappleby/smhasher/blob/0ff96f7835817a27d0487325b6c16033e2992eb5/src/MurmurHash3.cpp#L83-L87
        self.value ^= self.value >> 33;
        self.value = self.value.wrapping_mul(0xff51afd7ed558ccd);
        self.value ^= self.value >> 33;
        self.value = self.value.wrapping_mul(0xc4ceb9fe1a85ec53);
        self.value ^= self.value >> 33;
    }
}

pub fn sort_radix8(values: &mut [u8]) {
    if values.is_empty() {
        return;
    }
    const BITS: u8 = 8;
    let bin_mask = 1_u8 << (BITS - 1);
    sort_radix8_impl(values, bin_mask);
}

fn sort_radix8_impl(values: &mut [u8], bin_mask: u8) {
    let mut i_0bin_end = 0;
    let mut i_1bin_start = values.len();

    loop {
        if values[i_0bin_end] & bin_mask == 0 {
            i_0bin_end += 1;
        } else {
            i_1bin_start -= 1;
            values.swap(i_0bin_end, i_1bin_start);
        }

        if i_0bin_end == i_1bin_start {
            break;
        }
    }

    let next_bin_mask = bin_mask >> 1;
    if next_bin_mask == 0 {
        return;
    }

    if i_0bin_end > 0 {
        sort_radix8_impl(&mut values[..i_0bin_end], next_bin_mask);
    }

    if i_1bin_start < values.len() - 1 {
        sort_radix8_impl(&mut values[i_1bin_start..], next_bin_mask);
    }
}

#[cfg(test)]
mod tests {
    #[rustfmt::skip]
    const LOG2_CASES: &[(u64, u64)] = &[
        (0, 1),
        (1, 2), (1, 3),
        (2, 4), (2, 7),
        (3, 8), (3, 15),
        (4, 16), (4, 31),
        (38, 352371081216),
    ];

    #[test]
    fn log2() {
        for &(expected, x) in LOG2_CASES {
            assert_eq!(expected, super::log2(x));
        }
    }

    #[rustfmt::skip]
    const LOG10_CASES: &[(u64, u64)] = &[
        (0, 1),     (0, 9),
        (1, 10),    (1, 11),    (1, 99),
        (2, 100),   (2, 101),   (2, 999),
        (3, 1000),  (3, 1001),  (3, 9999),
        (4, 10000), (4, 10001), (4, 99999),
    ];

    #[test]
    fn log10() {
        for &(expected, x) in LOG10_CASES {
            assert_eq!(expected, super::log10(x));
        }
    }

    #[rustfmt::skip]
    const COUNT_DIGIT_CASES: &[(u64, u64)] = &[
        (1, 1),
        (2, 22),
        (3, 100), (3, 333),
        (4, 1000), (4, 4444),
        (5, 10000), (5, 55555),
        (6, 100000), (6, 666666),
        (7, 7777777), (7, 1000000),
        (8, 88888888), (8, 10000000),
        (9, 999999999), (9, 100000000),
    ];

    #[test]
    fn count_digits_with_log() {
        for &(expected, x) in COUNT_DIGIT_CASES {
            assert_eq!(expected, super::_count_digits_with_log(x));
        }
    }

    #[test]
    fn count_digits_fast() {
        for &(expected, x) in COUNT_DIGIT_CASES {
            assert_eq!(expected, super::_count_digits_fast(x));
        }
    }

    #[test]
    fn radix8_sort_1() {
        let mut values = [];

        super::sort_radix8(&mut values);

        assert_eq!([] as [u8; 0], values);
    }

    #[test]
    fn radix8_sort_2() {
        let mut values = [42];
        let expected = values;

        super::sort_radix8(&mut values);

        assert_eq!(expected, values);
    }

    #[test]
    fn radix8_sort_3() {
        let mut values = [0, 0, 1, 0, 1, 1, 1, 0];

        let mut expected = values;
        expected.sort();

        super::sort_radix8(&mut values);

        assert_eq!(expected, values);
    }

    #[test]
    fn radix8_sort_4() {
        let mut values: [_; 8] = std::array::from_fn(|_| rand::random());
        println!("{values:?}");

        let mut expected = values;
        expected.sort();

        super::sort_radix8(&mut values);

        assert_eq!(expected, values);
    }
}

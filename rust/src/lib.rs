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
    static POWERS_OF_TEN: [u64; 10] = [
        1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,
    ];

    let t = ((log2(v) + 1) * 1233) >> 12;
    t - (if v < POWERS_OF_TEN[t as usize] { 1 } else { 0 })
}

/// [Source](https://graphics.stanford.edu/~seander/bithacks.html#IntegerLogDeBruijn)
#[inline(always)]
fn log2(mut v: u64) -> u64 {
    static MULTIPLY_DEBRUIJN_BIT_POS: [u64; 32] = [
        0, 9, 1, 10, 13, 21, 2, 29, 11, 14, 16, 18, 22, 25, 3, 30, 8, 12, 20, 28, 15, 17, 24, 7,
        19, 27, 23, 6, 26, 5, 4, 31,
    ];

    v |= v >> 1; // first round down to one less than a power of 2 
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;

    MULTIPLY_DEBRUIJN_BIT_POS[0x07C4ACDD_u32.wrapping_mul(v as u32) as usize >> 27]
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
}

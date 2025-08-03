pub fn log2_ceil(n: usize) -> usize {
    let bit_width = if n <= 1 {
        1
    } else {
        (n - 1).ilog2() as usize + 1
    };
    bit_width
}
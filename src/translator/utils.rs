/// Generates a SyGuS bit-vector literal string.
/// e.g., to_sygus_bv(15, 4) -> "#b1111"
/// e.g., to_sygus_bv(0, 4) -> "#b0000"
pub fn bv(value: u64, width: usize) -> String {
    format!("#b{:0width$b}", value, width = width)
}

/// Generates a SyGuS bit-vector of all ones.
pub fn ones(width: usize) -> String {
    format!("#b{}", "1".repeat(width))
}

/// Generates a SyGuS bit-vector of all zeros.
pub fn zeros(width: usize) -> String {
    format!("#b{}", "0".repeat(width))
}

/// Safely calculates the required bit-width for a given number of elements.
/// Returns at least 1 bit width.
pub fn calculate_bit_width(num_elements: usize) -> usize {
    if num_elements <= 1 {
        1
    } else {
        (num_elements as f64).log2().ceil() as usize
    }
} 
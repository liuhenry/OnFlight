//! Fletcher 16 checksum as specified in the OnFlight Hub Developers Manual.

/// Compute Fletcher 16 checksum over a byte slice.
///
/// This matches the implementation from the dev manual:
/// ```ignore
/// uint16_t sum0 = 0, sum1 = 0;
/// for (size_t i = 0; i < len; i++) {
///     sum0 = (sum0 + data[i]) % 255;
///     sum1 = (sum1 + sum0) % 255;
/// }
/// return sum1 << 8 | sum0;
/// ```
pub fn fletcher16(data: &[u8]) -> u16 {
    let mut sum0: u16 = 0;
    let mut sum1: u16 = 0;
    for &byte in data {
        sum0 = (sum0 + byte as u16) % 255;
        sum1 = (sum1 + sum0) % 255;
    }
    (sum1 << 8) | sum0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert_eq!(fletcher16(&[]), 0);
    }

    #[test]
    fn test_single_byte() {
        // sum0 = 1, sum1 = 1 → 0x0101
        assert_eq!(fletcher16(&[1]), 0x0101);
    }

    #[test]
    fn test_known_value() {
        // "BF" header: sum0 = 66, sum1 = 66, then sum0 = 136, sum1 = 202
        // But let's just verify the actual computation
        let result = fletcher16(&[0x42, 0x46]);
        // sum0: (0+0x42)%255 = 66, then (66+0x46)%255 = 136
        // sum1: (0+66)%255 = 66, then (66+136)%255 = 202
        assert_eq!(result & 0xFF, 136); // sum0
        assert_eq!(result >> 8, 202);   // sum1
    }
}

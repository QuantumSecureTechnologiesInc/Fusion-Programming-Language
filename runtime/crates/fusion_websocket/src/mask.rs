//! XOR masking for WebSocket frames per RFC 6455.

/// Apply XOR masking to data with the given 4-byte mask key.
pub fn apply_mask(data: &mut [u8], mask: [u8; 4]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= mask[i % 4];
    }
}

/// Remove XOR masking from data with the given 4-byte mask key.
pub fn remove_mask(data: &mut [u8], mask: [u8; 4]) {
    // XOR is its own inverse
    apply_mask(data, mask)
}

/// Generate a random 4-byte mask key.
pub fn random_mask() -> [u8; 4] {
    use rand::RngCore;
    let mut mask = [0u8; 4];
    rand::thread_rng().fill_bytes(&mut mask);
    mask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_unmask_roundtrip() {
        let original = b"hello websocket world";
        let mut data = original.to_vec();
        let mask = [0x12, 0x34, 0x56, 0x78];

        apply_mask(&mut data, mask);
        assert_ne!(&data[..], original);

        remove_mask(&mut data, mask);
        assert_eq!(&data[..], original);
    }

    #[test]
    fn test_random_mask_unique() {
        let m1 = random_mask();
        let m2 = random_mask();
        // Extremely unlikely to be equal
        assert_ne!(m1, m2);
    }
}

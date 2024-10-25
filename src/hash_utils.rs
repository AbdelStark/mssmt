/// Converts a byte slice into an array of 32 bytes.
pub fn to_array(bytes: &[u8]) -> [u8; 32] {
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes[..32]);
    array
}

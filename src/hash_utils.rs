/// Converts a byte slice into an array of 32 bytes.
///
/// # Panics
///
/// This function will panic if the input slice is less than 32 bytes.
///
/// # Examples
///
/// ```
/// use mssmt::hash_utils::to_array;
///
/// let bytes = [1u8; 32];
/// let array = to_array(&bytes);
/// assert_eq!(array, bytes);
/// ```
///
/// ```should_panic
/// use mssmt::hash_utils::to_array;
///
/// // This will panic because the slice is less than 32 bytes
/// let bytes = [1u8; 16];
/// let _array = to_array(&bytes);
/// ```

pub fn to_array(bytes: &[u8]) -> [u8; 32] {
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes[..32]);
    array
}

//! Raw pointer helpers for C ABI entry points.

/// Converts a C byte pointer and length into a Rust slice.
pub(crate) fn bytes_from_raw<'a>(bytes: *const u8, len: usize) -> Option<&'a [u8]> {
    if len == 0 {
        return Some(&[]);
    }
    if bytes.is_null() {
        return None;
    }

    Some(unsafe { core::slice::from_raw_parts(bytes, len) })
}

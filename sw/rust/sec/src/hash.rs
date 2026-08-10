//! Keyed hashing helper shared by the integrity, audit, and channel modules.

/// Computes a keyed 32-byte tag over the given byte slices.
pub(crate) fn keyed_tag(key: &[u8; 32], parts: &[&[u8]]) -> [u8; 32] {
    let mut h = blake3::Hasher::new_keyed(key);
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

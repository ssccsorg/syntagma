use tagma_core::CoordSpace2;

use crate::coord_gen::CoordKey;
use crate::{CoordKV, CoordKVKey};

// ── Internal helpers ──────────────────────────────────────────────────────

fn vec_to_box(v: Vec<u8>) -> Box<[u8]> {
    v.into_boxed_slice()
}

fn box_to_vec(v: &[u8]) -> Vec<u8> {
    v.to_vec()
}

fn box_to_vec_owned(v: Box<[u8]>) -> Vec<u8> {
    v.into_vec()
}

// ── CoordKV2 ──────────────────────────────────────────────────────────────

/// A 2-byte-key KV store backed by [`CoordSpace2`] — the dense,
/// single-allocation array (119 MB).  Lookup cost is O(1).
///
/// Keys must be exactly 2 bytes.
///
/// # Trait implementations
///
/// | Trait | Methods |
/// |-------|---------|
/// | [`CoordKV`] | `insert`, `get`, `remove`, `contains_key` via `&str` |
/// | [`CoordKVKey<2>`] | `insert_by_coordkey`, `get_by_coordkey`, `remove_by_coordkey`, `contains_key_by_coordkey` via `CoordKey<2>` |
///
/// # Example
///
/// ```
/// use tagma_kv::{CoordKV, CoordKV2, CoordKVKey};
///
/// let mut kv = CoordKV2::new();
/// kv.insert("hi", b"world".to_vec());
/// assert_eq!(kv.get("hi"), Some(b"world".to_vec()));
/// ```
pub struct CoordKV2 {
    space: CoordSpace2<Box<[u8]>>,
    len: usize,
}

impl CoordKV2 {
    /// Creates an empty 2-byte-key store.
    ///
    /// Allocates 119 MB of zeroed memory (lazy-committed by the OS).
    pub fn new() -> Self {
        CoordKV2 {
            space: CoordSpace2::new(),
            len: 0,
        }
    }
}

impl CoordKV for CoordKV2 {
    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn clear(&mut self) {
        self.space.clear();
        self.len = 0;
    }

    fn insert(&mut self, key: &str, value: Vec<u8>) -> Option<Vec<u8>> {
        // Panics if key.len() != 2 (via CoordKey::from)
        let ck: CoordKey<2> = key.into();
        self.insert_by_coordkey(&ck, value)
    }

    fn get(&self, key: &str) -> Option<Vec<u8>> {
        if key.len() != 2 {
            return None;
        }
        let ck: CoordKey<2> = key.into();
        self.get_by_coordkey(&ck)
    }

    fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
        if key.len() != 2 {
            return None;
        }
        let ck: CoordKey<2> = key.into();
        self.remove_by_coordkey(&ck)
    }
}

impl CoordKVKey<2> for CoordKV2 {
    fn insert_by_coordkey(&mut self, key: &CoordKey<2>, value: Vec<u8>) -> Option<Vec<u8>> {
        let path = key.to_coord_path();
        let prev = self.space.place_path(&path, vec_to_box(value));
        if prev.is_none() {
            self.len += 1;
        }
        prev.map(box_to_vec_owned)
    }

    fn get_by_coordkey(&self, key: &CoordKey<2>) -> Option<Vec<u8>> {
        let path = key.to_coord_path();
        self.space.at_path(&path).map(|v| box_to_vec(v.as_ref()))
    }

    fn remove_by_coordkey(&mut self, key: &CoordKey<2>) -> Option<Vec<u8>> {
        let path = key.to_coord_path();
        let val = self.space.vacate_path(&path).map(box_to_vec_owned);
        if val.is_some() {
            self.len = self.len.saturating_sub(1);
        }
        val
    }
}

impl Default for CoordKV2 {
    fn default() -> Self {
        Self::new()
    }
}

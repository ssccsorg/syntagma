use tagma_core::CoordSpaceN;

use crate::coord_gen::CoordKey;
use crate::{CoordMap, CoordMapKey};

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

fn path_to_key_bytes<const N: usize>(path: &tagma_core::CoordPath<N>) -> [u8; N] {
    let mut key = [0u8; N];
    for (i, c) in path.coords().iter().enumerate() {
        key[i] = c.index() as u8;
    }
    key
}

// ── CoordMapN ──────────────────────────────────────────────────────────────

/// A fixed-N-byte-key map backed by [`CoordSpaceN`] — the sparse
/// tree for any depth `N`.
///
/// Keys must be exactly `N` bytes.  Lookup cost is O(N) via tree traversal.
///
/// # Trait implementations
///
/// | Trait | Methods |
/// |-------|---------|
/// | [`CoordMap`] | `insert`, `get`, `remove`, `contains_key` via `&str` |
/// | [`CoordMapKey<N>`] | `insert_by_coordkey`, `get_by_coordkey`, `remove_by_coordkey`, `contains_key_by_coordkey` via `CoordKey<N>` |
///
/// # Example
///
/// ```
/// use tagma_map::{CoordMap, CoordMapKey};
/// use tagma_map::coord_map_n::CoordMapN;
///
/// let mut kv = CoordMapN::<3>::new();
/// kv.insert("foo", b"bar".to_vec());
/// assert_eq!(kv.get("foo"), Some(b"bar".to_vec()));
/// ```
pub struct CoordMapN<const N: usize> {
    space: CoordSpaceN<N, Box<[u8]>>,
    len: usize,
}

impl<const N: usize> CoordMapN<N> {
    /// Creates an empty N-byte-key store.
    pub fn new() -> Self {
        CoordMapN {
            space: CoordSpaceN::new(),
            len: 0,
        }
    }

    /// Returns an iterator over `(key_bytes, value)` pairs in
    /// ascending coordinate order.
    pub fn iter(&self) -> CoordMapNIter<'_, N> {
        CoordMapNIter {
            inner: self.space.iter_tree(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<const N: usize> CoordMap for CoordMapN<N> {
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
        // Panics if key.len() != N (via CoordKey::from)
        let ck: CoordKey<N> = key.into();
        self.insert_by_coordkey(&ck, value)
    }

    fn get(&self, key: &str) -> Option<Vec<u8>> {
        if key.len() != N {
            return None;
        }
        let ck: CoordKey<N> = key.into();
        self.get_by_coordkey(&ck)
    }

    fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
        if key.len() != N {
            return None;
        }
        let ck: CoordKey<N> = key.into();
        self.remove_by_coordkey(&ck)
    }
}

impl<const N: usize> CoordMapKey<N> for CoordMapN<N> {
    fn insert_by_coordkey(&mut self, key: &CoordKey<N>, value: Vec<u8>) -> Option<Vec<u8>> {
        let path = key.to_coord_path();
        let prev = self.space.place_path(&path, vec_to_box(value));
        if prev.is_none() {
            self.len += 1;
        }
        prev.map(box_to_vec_owned)
    }

    fn get_by_coordkey(&self, key: &CoordKey<N>) -> Option<Vec<u8>> {
        let path = key.to_coord_path();
        self.space.at_path(&path).map(|v| box_to_vec(v.as_ref()))
    }

    fn remove_by_coordkey(&mut self, key: &CoordKey<N>) -> Option<Vec<u8>> {
        let path = key.to_coord_path();
        let val = self.space.vacate_path(&path).map(box_to_vec_owned);
        if val.is_some() {
            self.len = self.len.saturating_sub(1);
        }
        val
    }
}

impl<const N: usize> Default for CoordMapN<N> {
    fn default() -> Self {
        Self::new()
    }
}

// ── Iterator ──────────────────────────────────────────────────────────────

/// An iterator over [`CoordMapN<N>`] entries, yielding `(key_bytes, value)`.
pub struct CoordMapNIter<'a, const N: usize> {
    inner: tagma_core::coord_space_n::TreeIter<'a, N, Box<[u8]>>,
    _phantom: core::marker::PhantomData<[u8; N]>,
}

impl<'a, const N: usize> Iterator for CoordMapNIter<'a, N> {
    type Item = ([u8; N], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let (path, val) = self.inner.next()?;
        let key = path_to_key_bytes(&path);
        Some((key, val.as_ref()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

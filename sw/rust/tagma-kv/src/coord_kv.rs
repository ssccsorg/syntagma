use crate::coord_gen::CoordKey;

/// Core operations for any [`CoordKV`] implementation.
///
/// Mirrors [`HashMap`](std::collections::HashMap) where applicable:
/// `insert`, `get`, `remove`, `contains_key`, `len`, `is_empty`, `clear`.
///
/// Fixed-key variants additionally implement [`CoordKVKey`] for
/// [`CoordKey`]-based access (`_by_coordkey` suffix).
///
/// # Trait lego
///
/// | Trait | Methods | Scope |
/// |-------|---------|-------|
/// | [`CoordKV`] | `insert`, `get`, `remove`, `contains_key` via `&str` | all KV types |
/// | [`CoordKVKey<N>`] | `_by_coordkey` suffixed methods | fixed-key types only |
pub trait CoordKV {
    /// Returns the number of stored entries.
    fn len(&self) -> usize;

    /// Returns `true` if the store is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Removes all entries.
    fn clear(&mut self);

    /// Inserts a key-value pair.
    ///
    /// Returns the previous value if the key already existed, matching
    /// the [`HashMap::insert`](std::collections::HashMap::insert) contract.
    fn insert(&mut self, key: &str, value: Vec<u8>) -> Option<Vec<u8>>;

    /// Retrieves a value by key.
    fn get(&self, key: &str) -> Option<Vec<u8>>;

    /// Removes a key-value pair.  Returns the value if present.
    fn remove(&mut self, key: &str) -> Option<Vec<u8>>;

    /// Returns `true` if the store contains the given key.
    fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

/// [`CoordKey`]-based access for fixed-size-key KV stores.
///
/// Requires [`CoordKV`] and adds `_by_coordkey` methods.
pub trait CoordKVKey<const N: usize>: CoordKV {
    /// Inserts a key-value pair using a [`CoordKey<N>`].
    ///
    /// Returns the previous value if the key already existed.
    fn insert_by_coordkey(&mut self, key: &CoordKey<N>, value: Vec<u8>) -> Option<Vec<u8>>;

    /// Retrieves a value by [`CoordKey<N>`].
    fn get_by_coordkey(&self, key: &CoordKey<N>) -> Option<Vec<u8>>;

    /// Removes a key-value pair by [`CoordKey<N>`].
    fn remove_by_coordkey(&mut self, key: &CoordKey<N>) -> Option<Vec<u8>>;

    /// Returns `true` if the store contains the given [`CoordKey<N>`].
    fn contains_key_by_coordkey(&self, key: &CoordKey<N>) -> bool {
        self.get_by_coordkey(key).is_some()
    }
}

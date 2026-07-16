use crate::key::CoordKey;
use crate::map::CoordTreeMap;
use crate::path::CoordPath;
use alloc::vec::Vec;
use core::marker::PhantomData;

// ---------------------------------------------------------------------------
// CoordHashMap — HashMap-compatible, hash-free, collision-free map
// ---------------------------------------------------------------------------

/// A HashMap-compatible map backed by Tagma direct addressing.
///
/// `N` is the syllable depth (1 = 11,172 addresses, 6 = UUID-scale).
/// `K` is the key type, which must implement [`CoordKey<N>`].
///
/// # API compatibility
///
/// `new()`, `get(&K)`, `insert(K, V)`, `remove(&K)`, `contains_key(&K)`,
/// `len()`, `is_empty()`, `clear()`, `iter()`, `entry()` — all match
/// `std::collections::HashMap` signatures where `K` replaces `HashMap`'s
/// `K: Hash + Eq`.
///
/// # Collision model
///
/// Zero collisions at the storage level. For direct key types (`Coord`,
/// `u128`, `[u8; 16]`), collisions are zero end-to-end. For derived key
/// types (`&str`, `&[u8]`), collisions are probabilistic during the
/// hash-to-Coord conversion, matching `HashMap`'s model. At the storage
/// level, there are no bucket chains, no rehashing, and no load factor.
pub struct CoordHashMap<const N: usize, K: CoordKey<N>, V> {
    inner: CoordTreeMap<N, V>,
    _key: PhantomData<K>,
}

// ── Construction ────────────────────────────────────────────────────────

impl<const N: usize, K: CoordKey<N>, V> CoordHashMap<N, K, V> {
    /// Creates an empty `CoordHashMap`.
    #[inline]
    pub fn new() -> Self {
        CoordHashMap {
            inner: CoordTreeMap::new(),
            _key: PhantomData,
        }
    }

    /// Returns the number of entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the map contains no entries.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the maximum capacity for N=1 (11,172), or `None` for N>1.
    #[inline]
    pub fn capacity(&self) -> Option<usize> {
        self.inner.capacity()
    }
}

impl<const N: usize, K: CoordKey<N>, V> Default for CoordHashMap<N, K, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ── Core read / write ───────────────────────────────────────────────────

impl<const N: usize, K: CoordKey<N>, V> CoordHashMap<N, K, V> {
    /// Returns a reference to the value stored at `key`.
    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get_path(&key.to_path())
    }

    /// Returns a mutable reference to the value stored at `key`.
    #[inline]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.inner.get_path_mut(&key.to_path())
    }

    /// Returns `true` if the map contains an entry for `key`.
    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Inserts a value at `key`, returning the previous value if any.
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert_path(&key.to_path(), value)
    }

    /// Removes the value at `key`, returning it if present.
    #[inline]
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove_path(&key.to_path())
    }

    /// Removes all entries.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Collects all values, cloning each.
    /// For N=1, iterates all 11,172 slots; for N>1, walks the tree.
    pub fn values(&self) -> alloc::vec::Vec<V>
    where
        V: Clone,
    {
        self.iter_inner().map(|(_, v)| v.clone()).collect()
    }

    /// Extends the map from an iterator of key-value pairs.
    pub fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

// ── Iteration (internal) ───────────────────────────────────────────────

impl<const N: usize, K: CoordKey<N>, V> CoordHashMap<N, K, V> {
    fn iter_inner(&self) -> CoordHashMapIter<'_, N, V> {
        // For iteration we bypass the key type K and use raw CoordPath iteration.
        // This is only used internally for values() and similar bulk operations.
        CoordHashMapIter {
            inner: self.inner.iter_tree(),
        }
    }
}

struct CoordHashMapIter<'a, const N: usize, V> {
    inner: crate::map::TreeIter<'a, N, V>,
}

impl<'a, const N: usize, V> Iterator for CoordHashMapIter<'a, N, V> {
    type Item = (CoordPath<N>, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

// ── Entry API ───────────────────────────────────────────────────────────

impl<const N: usize, K: CoordKey<N>, V> CoordHashMap<N, K, V> {
    pub fn entry(&mut self, key: K) -> Entry<'_, N, K, V> {
        let path = key.to_path();
        if self.inner.get_path(&path).is_some() {
            Entry::Occupied(OccupiedEntry {
                map: self,
                path,
                _key: PhantomData,
            })
        } else {
            Entry::Vacant(VacantEntry {
                map: self,
                path,
                _key: PhantomData,
            })
        }
    }
}

pub enum Entry<'a, const N: usize, K: CoordKey<N>, V> {
    Occupied(OccupiedEntry<'a, N, K, V>),
    Vacant(VacantEntry<'a, N, K, V>),
}

pub struct OccupiedEntry<'a, const N: usize, K: CoordKey<N>, V> {
    map: &'a mut CoordHashMap<N, K, V>,
    path: CoordPath<N>,
    _key: PhantomData<K>,
}

pub struct VacantEntry<'a, const N: usize, K: CoordKey<N>, V> {
    map: &'a mut CoordHashMap<N, K, V>,
    path: CoordPath<N>,
    _key: PhantomData<K>,
}

impl<'a, const N: usize, K: CoordKey<N>, V> OccupiedEntry<'a, N, K, V> {
    pub fn get(&self) -> &V {
        unsafe { self.map.inner.get_path(&self.path).unwrap_unchecked() }
    }
    pub fn get_mut(&mut self) -> &mut V {
        unsafe { self.map.inner.get_path_mut(&self.path).unwrap_unchecked() }
    }
    pub fn insert(&mut self, value: V) -> V {
        unsafe { self.map.inner.insert_path(&self.path, value).unwrap_unchecked() }
    }
    pub fn remove_entry(self) -> V {
        unsafe { self.map.inner.remove_path(&self.path).unwrap_unchecked() }
    }
}

impl<'a, const N: usize, K: CoordKey<N>, V> VacantEntry<'a, N, K, V> {
    pub fn insert(self, value: V) -> &'a mut V {
        let _ = self.map.inner.insert_path(&self.path, value);
        unsafe { self.map.inner.get_path_mut(&self.path).unwrap_unchecked() }
    }
}

impl<'a, const N: usize, K: CoordKey<N>, V> Entry<'a, N, K, V> {
    pub fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert_with(|| default)
    }
    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> &'a mut V {
        match self {
            Entry::Occupied(e) => unsafe {
                e.map.inner.get_path_mut(&e.path).unwrap_unchecked()
            },
            Entry::Vacant(e) => e.insert(f()),
        }
    }
    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Self {
        if let Entry::Occupied(ref mut e) = self {
            f(e.get_mut());
        }
        self
    }
}

// ── FromIterator ────────────────────────────────────────────────────────

impl<const N: usize, K: CoordKey<N>, V> FromIterator<(K, V)> for CoordHashMap<N, K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut map = Self::new();
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
    }
}

// ── Debug ───────────────────────────────────────────────────────────────

impl<const N: usize, K: CoordKey<N>, V: core::fmt::Debug> core::fmt::Debug
    for CoordHashMap<N, K, V>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CoordHashMap")
            .field("N", &N)
            .field("len", &self.len())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Coord;
    use alloc::string::String;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn new_map_is_empty() {
        let map: CoordHashMap<1, &str, u32> = CoordHashMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn insert_and_get_str() {
        let mut map: CoordHashMap<1, &str, u32> = CoordHashMap::new();
        assert_eq!(map.insert("hello", 42), None);
        assert_eq!(map.get(&"hello"), Some(&42));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn insert_and_get_coord() {
        let mut map: CoordHashMap<1, Coord, u32> = CoordHashMap::new();
        let c = Coord::new(42).unwrap();
        assert_eq!(map.insert(c, 7), None);
        assert_eq!(map.get(&c), Some(&7));
    }

    #[test]
    fn insert_and_get_u128() {
        let mut map: CoordHashMap<6, u128, u32> = CoordHashMap::new();
        let key = 0x0123456789ABCDEF0123456789ABCDEFu128;
        assert_eq!(map.insert(key, 42), None);
        assert_eq!(map.get(&key), Some(&42));
    }

    #[test]
    fn insert_overwrite() {
        let mut map: CoordHashMap<1, &str, u32> = CoordHashMap::new();
        map.insert("key", 1);
        assert_eq!(map.insert("key", 2), Some(1));
        assert_eq!(map.get(&"key"), Some(&2));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove() {
        let mut map: CoordHashMap<1, &str, u32> = CoordHashMap::new();
        map.insert("key", 42);
        assert_eq!(map.remove(&"key"), Some(42));
        assert!(map.is_empty());
    }

    #[test]
    fn contains_key() {
        let mut map: CoordHashMap<1, &str, u32> = CoordHashMap::new();
        assert!(!map.contains_key(&"key"));
        map.insert("key", 1);
        assert!(map.contains_key(&"key"));
    }

    #[test]
    fn clear() {
        let mut map: CoordHashMap<1, &str, u32> = CoordHashMap::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.clear();
        assert!(map.is_empty());
    }

    #[test]
    fn entry_or_insert() {
        let mut map: CoordHashMap<1, &str, u32> = CoordHashMap::new();
        map.entry("key").or_insert(42);
        assert_eq!(map.get(&"key"), Some(&42));
        map.entry("key").or_insert(99);
        assert_eq!(map.get(&"key"), Some(&42));
    }

    #[test]
    fn entry_and_modify() {
        let mut map: CoordHashMap<1, &str, u32> = CoordHashMap::new();
        map.entry("key").and_modify(|v| *v += 1).or_insert(1);
        assert_eq!(map.get(&"key"), Some(&1));
        map.entry("key").and_modify(|v| *v += 1).or_insert(1);
        assert_eq!(map.get(&"key"), Some(&2));
    }

    #[test]
    fn from_iterator() {
        let entries = vec![("a", 1u32), ("b", 2), ("c", 3)];
        let map: CoordHashMap<1, &str, u32> = entries.into_iter().collect();
        assert_eq!(map.len(), 3);
        assert_eq!(map.get(&"a"), Some(&1));
    }

    #[test]
    fn default_is_empty() {
        let map: CoordHashMap<1, &str, u32> = Default::default();
        assert!(map.is_empty());
    }

    #[test]
    fn string_key_matches_str() {
        let mut map: CoordHashMap<1, String, u32> = CoordHashMap::new();
        map.insert("key".to_string(), 42);
        assert_eq!(map.get(&"key".to_string()), Some(&42));
    }
}

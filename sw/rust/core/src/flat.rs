use crate::coord::Coord;
use crate::path::CoordPath;

// ---------------------------------------------------------------------------
// CoordFlatMap: no_alloc, single-syllable direct-address table
// ---------------------------------------------------------------------------

/// A hash-less, collision-free, single-syllable address table with zero
/// heap allocation.
///
/// Backed by an inline `[Option<V>; 11172]` array (22 KB for `Option<()>`,
/// more for larger `V`). No allocator required — works in any `#[no_std]`
/// environment including bare-metal MCUs without a heap.
///
/// Every `Coord` is a direct array index:
///
/// ```text
/// slots[coord]  →  O(1), single array access
/// ```
#[derive(Clone, Debug)]
pub struct CoordFlatMap<V> {
    slots: [Option<V>; 11172],
    len: usize,
}

impl<V> CoordFlatMap<V> {
    // ── construction ────────────────────────────────────────────────────

    /// Creates an empty `CoordFlatMap`.
    ///
    /// All 11,172 slots are initialized to `None`.
    #[inline]
    pub fn new() -> Self {
        // SAFETY: [Option<V>; 11172] where all elements are None can be
        // represented as all-zeroes for any V (Option<V> has a niche).
        // This avoids running 11172 drop-and-replace instructions.
        let slots = unsafe { core::mem::zeroed() };
        CoordFlatMap { slots, len: 0 }
    }

    /// Returns the number of entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the map contains no entries.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the maximum capacity (always 11,172).
    #[inline]
    pub const fn capacity(&self) -> usize {
        11172
    }

    // ── read ────────────────────────────────────────────────────────────

    #[inline]
    fn slot(&self, coord: &Coord) -> &Option<V> {
        unsafe { self.slots.get_unchecked(coord.index() as usize) }
    }

    #[inline]
    fn slot_mut(&mut self, coord: &Coord) -> &mut Option<V> {
        unsafe { self.slots.get_unchecked_mut(coord.index() as usize) }
    }

    /// Returns a reference to the value at `coord`.
    #[inline]
    pub fn get(&self, coord: &Coord) -> Option<&V> {
        self.slot(coord).as_ref()
    }

    /// Returns a mutable reference to the value at `coord`.
    #[inline]
    pub fn get_mut(&mut self, coord: &Coord) -> Option<&mut V> {
        self.slot_mut(coord).as_mut()
    }

    /// Returns `true` if the map contains an entry for `coord`.
    #[inline]
    pub fn contains_key(&self, coord: &Coord) -> bool {
        self.slot(coord).is_some()
    }

    /// Returns a reference to the value at `path` (single-syllable path API).
    #[inline]
    pub fn get_path(&self, path: &CoordPath<1>) -> Option<&V> {
        self.get(&path.coords()[0])
    }

    // ── write ───────────────────────────────────────────────────────────

    /// Inserts a value at `coord`, returning the previous value if any.
    #[inline]
    pub fn insert(&mut self, coord: Coord, value: V) -> Option<V> {
        let slot = self.slot_mut(&coord);
        let old = slot.take();
        *slot = Some(value);
        if old.is_none() {
            self.len += 1;
        }
        old
    }

    /// Inserts a value at `path` (single-syllable path API).
    #[inline]
    pub fn insert_path(&mut self, path: &CoordPath<1>, value: V) -> Option<V> {
        self.insert(path.coords()[0], value)
    }

    /// Removes the value at `coord`, returning it if present.
    #[inline]
    pub fn remove(&mut self, coord: &Coord) -> Option<V> {
        let slot = self.slot_mut(coord);
        let old = slot.take();
        if old.is_some() {
            self.len -= 1;
        }
        old
    }

    /// Removes the value at `path` (single-syllable path API).
    #[inline]
    pub fn remove_path(&mut self, path: &CoordPath<1>) -> Option<V> {
        self.remove(&path.coords()[0])
    }

    /// Clears the map, removing all entries.
    pub fn clear(&mut self) {
        for slot in self.slots.iter_mut() {
            *slot = None;
        }
        self.len = 0;
    }

    // ── iteration ──────────────────────────────────────────────────────

    pub fn iter(&self) -> FlatIter<'_, V> {
        FlatIter {
            slots: self.slots.iter(),
            idx: 0,
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = Coord> + '_ {
        self.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.iter().map(|(_, v)| v)
    }

    pub fn retain<F: FnMut(Coord, &mut V) -> bool>(&mut self, mut f: F) {
        for (idx, slot) in self.slots.iter_mut().enumerate() {
            if let Some(coord) = Coord::new(idx as u16) {
                if let Some(val) = slot.as_mut() {
                    if !f(coord, val) {
                        *slot = None;
                        self.len -= 1;
                    }
                }
            }
        }
    }

    pub fn iter_mut(&mut self) -> FlatIterMut<'_, V> {
        FlatIterMut {
            slots: self.slots.iter_mut(),
            idx: 0,
        }
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> + '_ {
        self.iter_mut().map(|(_, v)| v)
    }

    pub fn drain(&mut self) -> FlatDrain<'_, V> {
        FlatDrain { map: self, idx: 0 }
    }

    // ── entry API ──────────────────────────────────────────────────────

    pub fn entry(&mut self, coord: Coord) -> FlatEntry<'_, V> {
        if self.contains_key(&coord) {
            FlatEntry::Occupied(FlatOccupiedEntry { map: self, coord })
        } else {
            FlatEntry::Vacant(FlatVacantEntry { map: self, coord })
        }
    }
}

impl<V> Default for CoordFlatMap<V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ── Iterators ─────────────────────────────────────────

pub struct FlatIter<'a, V> {
    slots: core::slice::Iter<'a, Option<V>>,
    idx: u16,
}

impl<'a, V> Iterator for FlatIter<'a, V> {
    type Item = (Coord, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        for slot in self.slots.by_ref() {
            let coord = Coord::new(self.idx).unwrap();
            self.idx += 1;
            if let Some(val) = slot.as_ref() {
                return Some((coord, val));
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.slots.len()))
    }
}

pub struct FlatDrain<'a, V> {
    map: &'a mut CoordFlatMap<V>,
    idx: u16,
}

impl<'a, V> Iterator for FlatDrain<'a, V> {
    type Item = (Coord, V);
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < 11172 {
            let coord = Coord::new(self.idx).unwrap();
            self.idx += 1;
            if let Some(val) = self.map.remove(&coord) {
                return Some((coord, val));
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(11172 - self.idx as usize))
    }
}

impl<'a, V> Drop for FlatDrain<'a, V> {
    fn drop(&mut self) {
        while self.idx < 11172 {
            let coord = Coord::new(self.idx).unwrap();
            self.idx += 1;
            self.map.remove(&coord);
        }
    }
}

// ── IterMut ────────────────────────────────────────────

pub struct FlatIterMut<'a, V> {
    slots: core::slice::IterMut<'a, Option<V>>,
    idx: u16,
}

impl<'a, V> Iterator for FlatIterMut<'a, V> {
    type Item = (Coord, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        for slot in self.slots.by_ref() {
            let coord = Coord::new(self.idx).unwrap();
            self.idx += 1;
            if let Some(val) = slot.as_mut() {
                return Some((coord, val));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.slots.len()))
    }
}

// ── Entry API ──────────────────────────────────────────

pub enum FlatEntry<'a, V> {
    Occupied(FlatOccupiedEntry<'a, V>),
    Vacant(FlatVacantEntry<'a, V>),
}

pub struct FlatOccupiedEntry<'a, V> {
    map: &'a mut CoordFlatMap<V>,
    coord: Coord,
}

impl<'a, V> FlatOccupiedEntry<'a, V> {
    pub fn key(&self) -> Coord {
        self.coord
    }
    pub fn get(&self) -> &V {
        unsafe { self.map.get(&self.coord).unwrap_unchecked() }
    }
    pub fn get_mut(&mut self) -> &mut V {
        unsafe { self.map.get_mut(&self.coord).unwrap_unchecked() }
    }
    pub fn insert(&mut self, value: V) -> V {
        unsafe { self.map.insert(self.coord, value).unwrap_unchecked() }
    }
    pub fn remove_entry(self) -> V {
        unsafe { self.map.remove(&self.coord).unwrap_unchecked() }
    }
}

pub struct FlatVacantEntry<'a, V> {
    map: &'a mut CoordFlatMap<V>,
    coord: Coord,
}

impl<'a, V> FlatVacantEntry<'a, V> {
    pub fn key(&self) -> Coord {
        self.coord
    }
    pub fn into_key(self) -> Coord {
        self.coord
    }
    pub fn insert(self, value: V) -> &'a mut V {
        let _ = self.map.insert(self.coord, value);
        unsafe { self.map.get_mut(&self.coord).unwrap_unchecked() }
    }
}

impl<'a, V> FlatEntry<'a, V> {
    pub fn key(&self) -> Coord {
        match self {
            FlatEntry::Occupied(e) => e.key(),
            FlatEntry::Vacant(e) => e.key(),
        }
    }
    pub fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert_with(|| default)
    }
    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> &'a mut V {
        match self {
            FlatEntry::Occupied(e) => unsafe { e.map.get_mut(&e.coord).unwrap_unchecked() },
            FlatEntry::Vacant(e) => e.insert(f()),
        }
    }
    pub fn or_insert_with_key<F: FnOnce(Coord) -> V>(self, f: F) -> &'a mut V {
        match self {
            FlatEntry::Occupied(e) => unsafe { e.map.get_mut(&e.coord).unwrap_unchecked() },
            FlatEntry::Vacant(e) => {
                let v = f(e.coord);
                e.insert(v)
            }
        }
    }
    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Self {
        if let FlatEntry::Occupied(ref mut e) = self {
            f(e.get_mut());
        }
        self
    }
}

// ── FromIterator / IntoIterator ────────────────────────

impl<V> FromIterator<(Coord, V)> for CoordFlatMap<V> {
    fn from_iter<I: IntoIterator<Item = (Coord, V)>>(iter: I) -> Self {
        let mut map = Self::new();
        for (coord, value) in iter {
            map.insert(coord, value);
        }
        map
    }
}

impl<V> IntoIterator for CoordFlatMap<V> {
    type Item = (Coord, V);
    type IntoIter = FlatIntoIter<V>;
    fn into_iter(mut self) -> Self::IntoIter {
        self.len = 0;
        FlatIntoIter { map: self, idx: 0 }
    }
}

pub struct FlatIntoIter<V> {
    map: CoordFlatMap<V>,
    idx: u16,
}

impl<V> Iterator for FlatIntoIter<V> {
    type Item = (Coord, V);
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < 11172 {
            let coord = Coord::new(self.idx).unwrap();
            self.idx += 1;
            let slot = unsafe { self.map.slots.get_unchecked_mut(coord.index() as usize) };
            if let Some(val) = slot.take() {
                return Some((coord, val));
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(11172 - self.idx as usize))
    }
}

impl<'a, V> IntoIterator for &'a CoordFlatMap<V> {
    type Item = (Coord, &'a V);
    type IntoIter = FlatIter<'a, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// ── Index ──────────────────────────────────────────────

impl<V> core::ops::Index<Coord> for CoordFlatMap<V> {
    type Output = V;
    fn index(&self, coord: Coord) -> &V {
        self.get(&coord)
            .expect("CoordFlatMap::index: key not present")
    }
}

impl<V> core::ops::IndexMut<Coord> for CoordFlatMap<V> {
    fn index_mut(&mut self, coord: Coord) -> &mut V {
        self.get_mut(&coord)
            .expect("CoordFlatMap::index_mut: key not present")
    }
}

// ── Eq ─────────────────────────────────────────────────

impl<V: PartialEq> PartialEq for CoordFlatMap<V> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.slots == other.slots
    }
}
impl<V: PartialEq> Eq for CoordFlatMap<V> {}

// ── Type alias ─────────────────────────────────────────

/// Default single-syllable address table. No allocator required.
pub type CoordMap<V> = CoordFlatMap<V>;

/// 1-syllable: 11,172 identifiers. No allocator required.
pub type CoordMap1<V> = CoordFlatMap<V>;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn new_map_is_empty() {
        let map: CoordFlatMap<u32> = CoordFlatMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert_eq!(map.capacity(), 11172);
    }

    #[test]
    fn insert_and_get() {
        let mut map = CoordFlatMap::new();
        let c = Coord::new(0).unwrap();
        assert_eq!(map.insert(c, 42), None);
        assert_eq!(map.get(&c), Some(&42));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn insert_overwrite() {
        let mut map = CoordFlatMap::new();
        let c = Coord::new(0).unwrap();
        map.insert(c, 1);
        assert_eq!(map.insert(c, 2), Some(1));
        assert_eq!(map.get(&c), Some(&2));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove() {
        let mut map = CoordFlatMap::new();
        let c = Coord::new(0).unwrap();
        map.insert(c, 42);
        assert_eq!(map.remove(&c), Some(42));
        assert!(map.is_empty());
    }

    #[test]
    fn contains_key() {
        let mut map = CoordFlatMap::new();
        let c = Coord::new(0).unwrap();
        assert!(!map.contains_key(&c));
        map.insert(c, ());
        assert!(map.contains_key(&c));
    }

    #[test]
    fn clear() {
        let mut map = CoordFlatMap::new();
        map.insert(Coord::new(0).unwrap(), 1);
        map.insert(Coord::new(100).unwrap(), 2);
        map.clear();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn iter_non_empty() {
        let mut map = CoordFlatMap::new();
        let c1 = Coord::new(0).unwrap();
        let c2 = Coord::new(9999).unwrap();
        map.insert(c1, 10);
        map.insert(c2, 20);
        let entries: Vec<_> = map.iter().collect();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&(c1, &10)));
        assert!(entries.contains(&(c2, &20)));
    }

    #[test]
    fn into_iter() {
        let mut map = CoordFlatMap::new();
        let c = Coord::new(42).unwrap();
        map.insert(c, "hello");
        let collected: Vec<_> = map.into_iter().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].0, c);
        assert_eq!(collected[0].1, "hello");
    }

    #[test]
    fn from_iterator() {
        let pairs: Vec<_> = (0..5u16).map(|i| (Coord::new(i).unwrap(), i)).collect();
        let map: CoordFlatMap<u16> = pairs.into_iter().collect();
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn entry_or_insert() {
        let mut map = CoordFlatMap::new();
        let c = Coord::new(0).unwrap();
        map.entry(c).or_insert(42);
        assert_eq!(map.get(&c), Some(&42));
        map.entry(c).or_insert(99);
        assert_eq!(map.get(&c), Some(&42));
    }

    #[test]
    fn index_trait() {
        let mut map = CoordFlatMap::new();
        let c = Coord::new(5).unwrap();
        map.insert(c, 42);
        assert_eq!(map[c], 42);
        map[c] = 99;
        assert_eq!(map[c], 99);
    }

    #[test]
    fn iter_mut() {
        let mut map = CoordFlatMap::new();
        let c = Coord::new(5).unwrap();
        map.insert(c, 1);
        for (_, v) in map.iter_mut() {
            *v += 1;
        }
        assert_eq!(map.get(&c), Some(&2));
    }

    #[test]
    fn values_mut() {
        let mut map = CoordFlatMap::new();
        map.insert(Coord::new(0).unwrap(), 10);
        map.insert(Coord::new(1).unwrap(), 20);
        for v in map.values_mut() {
            *v *= 2;
        }
        assert_eq!(map.get(&Coord::new(0).unwrap()), Some(&20));
        assert_eq!(map.get(&Coord::new(1).unwrap()), Some(&40));
    }

    #[test]
    fn retain() {
        let mut map = CoordFlatMap::new();
        map.insert(Coord::new(0).unwrap(), 1);
        map.insert(Coord::new(1).unwrap(), 2);
        map.retain(|_, v| *v > 1);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn drain() {
        let mut map = CoordFlatMap::new();
        map.insert(Coord::new(0).unwrap(), 1);
        map.insert(Coord::new(1).unwrap(), 2);
        let drained: Vec<_> = map.drain().collect();
        assert_eq!(drained.len(), 2);
        assert!(map.is_empty());
    }

    #[test]
    fn path_api() {
        let mut map = CoordFlatMap::new();
        let c = Coord::new(42).unwrap();
        map.insert_path(&CoordPath::new([c]), 100);
        assert_eq!(map.get_path(&CoordPath::new([c])), Some(&100));
        assert_eq!(map.remove_path(&CoordPath::new([c])), Some(100));
        assert!(map.is_empty());
    }

    #[test]
    fn insert_11172_values() {
        let mut map = CoordFlatMap::new();
        for i in 0u16..11172 {
            assert_eq!(map.insert(Coord::new(i).unwrap(), i), None);
        }
        assert_eq!(map.len(), 11172);
        for i in 0u16..11172 {
            assert_eq!(map.get(&Coord::new(i).unwrap()), Some(&i));
        }
    }

    #[test]
    fn eq() {
        let mut a = CoordFlatMap::new();
        let mut b = CoordFlatMap::new();
        a.insert(Coord::new(0).unwrap(), 1);
        b.insert(Coord::new(0).unwrap(), 1);
        assert_eq!(a, b);
        b.insert(Coord::new(1).unwrap(), 2);
        assert_ne!(a, b);
    }

    #[test]
    fn default() {
        let map: CoordFlatMap<u32> = Default::default();
        assert!(map.is_empty());
    }
}

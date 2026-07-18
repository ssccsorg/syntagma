use crate::coord::Coord;
use crate::coord_path::CoordPath;

// ---------------------------------------------------------------------------
// CoordSpace: no_alloc, single-syllable direct-address table
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
pub struct CoordSpace<V> {
    slots: [Option<V>; 11172],
    len: usize,
}

impl<V> CoordSpace<V> {
    // ── construction ────────────────────────────────────────────────────

    /// Creates an empty `CoordSpace`.
    ///
    /// All 11,172 slots are initialized to `None`.
    #[inline]
    pub fn new() -> Self {
        // SAFETY: [Option<V>; 11172] where all elements are None can be
        // represented as all-zeroes for any V (Option<V> has a niche).
        // This avoids running 11172 drop-and-replace instructions.
        let slots = unsafe { core::mem::zeroed() };
        CoordSpace { slots, len: 0 }
    }

    /// Returns the number of entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the space contains no entries.
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
    pub fn at(&self, coord: &Coord) -> Option<&V> {
        self.slot(coord).as_ref()
    }

    /// Returns a mutable reference to the value at `coord`.
    #[inline]
    pub fn at_mut(&mut self, coord: &Coord) -> Option<&mut V> {
        self.slot_mut(coord).as_mut()
    }

    /// Returns the coordinate-value pair for `coord`.
    #[inline]
    pub fn get_entry(&self, coord: &Coord) -> Option<(Coord, &V)> {
        self.slot(coord).as_ref().map(|v| (*coord, v))
    }

    /// Returns `true` if the space contains an entry for `coord`.
    #[inline]
    pub fn occupied(&self, coord: &Coord) -> bool {
        self.slot(coord).is_some()
    }

    /// Returns a reference to the value at `path` (single-syllable path API).
    #[inline]
    pub fn at_path(&self, path: &CoordPath<1>) -> Option<&V> {
        self.at(&path.coords()[0])
    }

    // ── write ───────────────────────────────────────────────────────────

    /// Inserts a value at `coord`, returning the previous value if any.
    #[inline]
    pub fn place(&mut self, coord: Coord, value: V) -> Option<V> {
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
    pub fn place_path(&mut self, path: &CoordPath<1>, value: V) -> Option<V> {
        self.place(path.coords()[0], value)
    }

    /// Removes the value at `coord`, returning it if present.
    #[inline]
    pub fn vacate(&mut self, coord: &Coord) -> Option<V> {
        let slot = self.slot_mut(coord);
        let old = slot.take();
        if old.is_some() {
            self.len -= 1;
        }
        old
    }

    /// Removes the value at `path` (single-syllable path API).
    #[inline]
    pub fn vacate_path(&mut self, path: &CoordPath<1>) -> Option<V> {
        self.vacate(&path.coords()[0])
    }

    /// Clears the space, removing all entries.
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

    pub fn coords(&self) -> impl Iterator<Item = Coord> + '_ {
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
        FlatDrain {
            space: self,
            idx: 0,
        }
    }

    // ── entry API ──────────────────────────────────────────────────────

    pub fn entry(&mut self, coord: Coord) -> FlatEntry<'_, V> {
        if self.occupied(&coord) {
            FlatEntry::Occupied(FlatOccupiedEntry { space: self, coord })
        } else {
            FlatEntry::Vacant(FlatVacantEntry { space: self, coord })
        }
    }
}

impl<V> Default for CoordSpace<V> {
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
    space: &'a mut CoordSpace<V>,
    idx: u16,
}

impl<'a, V> Iterator for FlatDrain<'a, V> {
    type Item = (Coord, V);
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < 11172 {
            let coord = Coord::new(self.idx).unwrap();
            self.idx += 1;
            if let Some(val) = self.space.vacate(&coord) {
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
            self.space.vacate(&coord);
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
    space: &'a mut CoordSpace<V>,
    coord: Coord,
}

impl<'a, V> FlatOccupiedEntry<'a, V> {
    pub fn coord(&self) -> Coord {
        self.coord
    }
    pub fn at(&self) -> &V {
        unsafe { self.space.at(&self.coord).unwrap_unchecked() }
    }
    pub fn at_mut(&mut self) -> &mut V {
        unsafe { self.space.at_mut(&self.coord).unwrap_unchecked() }
    }
    pub fn place(&mut self, value: V) -> V {
        unsafe { self.space.place(self.coord, value).unwrap_unchecked() }
    }
    pub fn remove_entry(self) -> V {
        unsafe { self.space.vacate(&self.coord).unwrap_unchecked() }
    }
}

pub struct FlatVacantEntry<'a, V> {
    space: &'a mut CoordSpace<V>,
    coord: Coord,
}

impl<'a, V> FlatVacantEntry<'a, V> {
    pub fn key(&self) -> Coord {
        self.coord
    }
    pub fn into_key(self) -> Coord {
        self.coord
    }
    pub fn place(self, value: V) -> &'a mut V {
        let _ = self.space.place(self.coord, value);
        unsafe { self.space.at_mut(&self.coord).unwrap_unchecked() }
    }
}

impl<'a, V> FlatEntry<'a, V> {
    pub fn key(&self) -> Coord {
        match self {
            FlatEntry::Occupied(e) => e.coord(),
            FlatEntry::Vacant(e) => e.key(),
        }
    }
    pub fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert_with(|| default)
    }
    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> &'a mut V {
        match self {
            FlatEntry::Occupied(e) => unsafe { e.space.at_mut(&e.coord).unwrap_unchecked() },
            FlatEntry::Vacant(e) => e.place(f()),
        }
    }
    pub fn or_insert_with_key<F: FnOnce(Coord) -> V>(self, f: F) -> &'a mut V {
        match self {
            FlatEntry::Occupied(e) => unsafe { e.space.at_mut(&e.coord).unwrap_unchecked() },
            FlatEntry::Vacant(e) => {
                let v = f(e.coord);
                e.place(v)
            }
        }
    }
    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Self {
        if let FlatEntry::Occupied(ref mut e) = self {
            f(e.at_mut());
        }
        self
    }
}

// ── FromIterator / IntoIterator ────────────────────────

impl<V> FromIterator<(Coord, V)> for CoordSpace<V> {
    fn from_iter<I: IntoIterator<Item = (Coord, V)>>(iter: I) -> Self {
        let mut space = Self::new();
        for (coord, value) in iter {
            space.place(coord, value);
        }
        space
    }
}

impl<V> IntoIterator for CoordSpace<V> {
    type Item = (Coord, V);
    type IntoIter = FlatIntoIter<V>;
    fn into_iter(mut self) -> Self::IntoIter {
        self.len = 0;
        FlatIntoIter { map: self, idx: 0 }
    }
}

pub struct FlatIntoIter<V> {
    map: CoordSpace<V>,
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

impl<'a, V> IntoIterator for &'a CoordSpace<V> {
    type Item = (Coord, &'a V);
    type IntoIter = FlatIter<'a, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// ── Index ──────────────────────────────────────────────

impl<V> core::ops::Index<Coord> for CoordSpace<V> {
    type Output = V;
    fn index(&self, coord: Coord) -> &V {
        self.at(&coord).expect("CoordSpace::index: key not present")
    }
}

impl<V> core::ops::IndexMut<Coord> for CoordSpace<V> {
    fn index_mut(&mut self, coord: Coord) -> &mut V {
        self.at_mut(&coord)
            .expect("CoordSpace::index_mut: key not present")
    }
}

// ── Eq ─────────────────────────────────────────────────

impl<V: PartialEq> PartialEq for CoordSpace<V> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.slots == other.slots
    }
}
impl<V: PartialEq> Eq for CoordSpace<V> {}

// ── Type alias ─────────────────────────────────────────

/// Default single-syllable address table. No allocator required.
pub type CoordSpaceN<V> = CoordSpace<V>;

/// 1-syllable: 11,172 identifiers. No allocator required.
pub type CoordSpace1<V> = CoordSpace<V>;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn new_map_is_empty() {
        let map: CoordSpace<u32> = CoordSpace::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert_eq!(map.capacity(), 11172);
    }

    #[test]
    fn insert_and_get() {
        let mut map = CoordSpace::new();
        let c = Coord::new(0).unwrap();
        assert_eq!(map.place(c, 42), None);
        assert_eq!(map.at(&c), Some(&42));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn insert_overwrite() {
        let mut map = CoordSpace::new();
        let c = Coord::new(0).unwrap();
        map.place(c, 1);
        assert_eq!(map.place(c, 2), Some(1));
        assert_eq!(map.at(&c), Some(&2));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn vacate() {
        let mut map = CoordSpace::new();
        let c = Coord::new(0).unwrap();
        map.place(c, 42);
        assert_eq!(map.vacate(&c), Some(42));
        assert!(map.is_empty());
    }

    #[test]
    fn contains_key() {
        let mut map = CoordSpace::new();
        let c = Coord::new(0).unwrap();
        assert!(!map.occupied(&c));
        map.place(c, ());
        assert!(map.occupied(&c));
    }

    #[test]
    fn clear() {
        let mut map = CoordSpace::new();
        map.place(Coord::new(0).unwrap(), 1);
        map.place(Coord::new(100).unwrap(), 2);
        map.clear();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn iter_non_empty() {
        let mut map = CoordSpace::new();
        let c1 = Coord::new(0).unwrap();
        let c2 = Coord::new(9999).unwrap();
        map.place(c1, 10);
        map.place(c2, 20);
        let entries: Vec<_> = map.iter().collect();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&(c1, &10)));
        assert!(entries.contains(&(c2, &20)));
    }

    #[test]
    fn into_iter() {
        let mut map = CoordSpace::new();
        let c = Coord::new(42).unwrap();
        map.place(c, "hello");
        let collected: Vec<_> = map.into_iter().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].0, c);
        assert_eq!(collected[0].1, "hello");
    }

    #[test]
    fn from_iterator() {
        let pairs: Vec<_> = (0..5u16).map(|i| (Coord::new(i).unwrap(), i)).collect();
        let map: CoordSpace<u16> = pairs.into_iter().collect();
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn entry_or_insert() {
        let mut map = CoordSpace::new();
        let c = Coord::new(0).unwrap();
        map.entry(c).or_insert(42);
        assert_eq!(map.at(&c), Some(&42));
        map.entry(c).or_insert(99);
        assert_eq!(map.at(&c), Some(&42));
    }

    #[test]
    fn index_trait() {
        let mut map = CoordSpace::new();
        let c = Coord::new(5).unwrap();
        map.place(c, 42);
        assert_eq!(map[c], 42);
        map[c] = 99;
        assert_eq!(map[c], 99);
    }

    #[test]
    fn iter_mut() {
        let mut map = CoordSpace::new();
        let c = Coord::new(5).unwrap();
        map.place(c, 1);
        for (_, v) in map.iter_mut() {
            *v += 1;
        }
        assert_eq!(map.at(&c), Some(&2));
    }

    #[test]
    fn values_mut() {
        let mut map = CoordSpace::new();
        map.place(Coord::new(0).unwrap(), 10);
        map.place(Coord::new(1).unwrap(), 20);
        for v in map.values_mut() {
            *v *= 2;
        }
        assert_eq!(map.at(&Coord::new(0).unwrap()), Some(&20));
        assert_eq!(map.at(&Coord::new(1).unwrap()), Some(&40));
    }

    #[test]
    fn retain() {
        let mut map = CoordSpace::new();
        map.place(Coord::new(0).unwrap(), 1);
        map.place(Coord::new(1).unwrap(), 2);
        map.retain(|_, v| *v > 1);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn drain() {
        let mut map = CoordSpace::new();
        map.place(Coord::new(0).unwrap(), 1);
        map.place(Coord::new(1).unwrap(), 2);
        let drained: Vec<_> = map.drain().collect();
        assert_eq!(drained.len(), 2);
        assert!(map.is_empty());
    }

    #[test]
    fn path_api() {
        let mut map = CoordSpace::new();
        let c = Coord::new(42).unwrap();
        map.place_path(&CoordPath::new([c]), 100);
        assert_eq!(map.at_path(&CoordPath::new([c])), Some(&100));
        assert_eq!(map.vacate_path(&CoordPath::new([c])), Some(100));
        assert!(map.is_empty());
    }

    #[test]
    fn insert_11172_values() {
        let mut map = CoordSpace::new();
        for i in 0u16..11172 {
            assert_eq!(map.place(Coord::new(i).unwrap(), i), None);
        }
        assert_eq!(map.len(), 11172);
        for i in 0u16..11172 {
            assert_eq!(map.at(&Coord::new(i).unwrap()), Some(&i));
        }
    }

    #[test]
    fn eq() {
        let mut a = CoordSpace::new();
        let mut b = CoordSpace::new();
        a.place(Coord::new(0).unwrap(), 1);
        b.place(Coord::new(0).unwrap(), 1);
        assert_eq!(a, b);
        b.place(Coord::new(1).unwrap(), 2);
        assert_ne!(a, b);
    }

    #[test]
    fn default() {
        let map: CoordSpace<u32> = Default::default();
        assert!(map.is_empty());
    }
}

use crate::coord::Coord;
use crate::path::CoordPath;
use alloc::boxed::Box;
use alloc::vec::Vec;

// ---------------------------------------------------------------------------
// TreeMap: hash-less, collision-free N-level address table (N>1)
// ---------------------------------------------------------------------------

/// A hash-less, collision-free, N-level address table indexed by [`CoordPath`]
/// for N > 1. Requires a heap allocator.
///
/// For single-syllable addressing without heap allocation, use [`CoordMap`].
///
/// # Depth
///
/// | `N`  | Identifier space | Typical use |
/// |------|-----------------|-------------|
/// | 2    | 1.25 × 10⁸     | Small KV |
/// | 6    | 1.94 × 10²⁴    | UUID-scale |
/// | 12   | 2.41 × 10⁶⁷    | Between UUID and SHA-256 |
/// | 19   | 1.94 × 10⁷⁷    | SHA-256-scale (2²⁵⁶) |
///
/// # Hash-less principle
///
/// Every level is a direct array index into 11,172 slots:
///
/// ```text
/// N=2:  root.branch[coord₀].leaf[coord₁]       → 2 array accesses
/// N=6:  root.branch[coord₀]...[leaf][coord₅]   → 6 array accesses
/// ```
///
/// No hashing, no collision resolution at any depth.
pub struct TreeMap<const N: usize, V> {
    root: Node<V>,
    len: usize,
}

/// A single node in the tree — zero unsafe code.
///
/// - `Leaf`: boxed slice of 11,172 `Option<V>` slots.
/// - `Branch`: boxed slice of 11,172 `Option<Box<Node<V>>>` slots.
enum Node<V> {
    Leaf(Box<[Option<V>]>),
    Branch(Box<[Option<Box<Node<V>>>]>),
}

impl<V> Node<V> {
    #[inline]
    fn new_leaf() -> Self {
        Node::Leaf((0..11172).map(|_| None).collect::<Vec<_>>().into_boxed_slice())
    }

    #[inline]
    fn new_branch() -> Self {
        Node::Branch((0..11172).map(|_| None).collect::<Vec<_>>().into_boxed_slice())
    }

    #[inline]
    fn get_value(&self, index: usize) -> Option<&V> {
        match self {
            Node::Leaf(s) => s[index].as_ref(),
            Node::Branch(_) => unreachable!(),
        }
    }

    #[inline]
    fn get_value_mut(&mut self, index: usize) -> Option<&mut V> {
        match self {
            Node::Leaf(s) => s[index].as_mut(),
            Node::Branch(_) => unreachable!(),
        }
    }

    #[inline]
    fn take_value(&mut self, index: usize) -> Option<V> {
        match self {
            Node::Leaf(s) => s[index].take(),
            Node::Branch(_) => unreachable!(),
        }
    }

    #[inline]
    fn set_value(&mut self, index: usize, value: V) -> Option<V> {
        match self {
            Node::Leaf(s) => {
                let old = s[index].take();
                s[index] = Some(value);
                old
            }
            Node::Branch(_) => unreachable!(),
        }
    }

    #[inline]
    fn get_child(&self, index: usize) -> Option<&Node<V>> {
        match self {
            Node::Branch(s) => s[index].as_deref(),
            Node::Leaf(_) => unreachable!(),
        }
    }

    #[inline]
    fn get_child_mut_existing(&mut self, index: usize) -> Option<&mut Node<V>> {
        match self {
            Node::Branch(s) => s[index].as_deref_mut(),
            Node::Leaf(_) => unreachable!(),
        }
    }

    #[inline]
    fn get_child_mut(&mut self, index: usize, is_last: bool) -> &mut Node<V> {
        match self {
            Node::Branch(s) => {
                let slot = &mut s[index];
                slot.get_or_insert_with(|| {
                    if is_last {
                        Box::new(Node::new_leaf())
                    } else {
                        Box::new(Node::new_branch())
                    }
                })
            }
            Node::Leaf(_) => unreachable!(),
        }
    }
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

impl<const N: usize, V> TreeMap<N, V> {
    /// Creates an empty `TreeMap`.
    ///
    /// For `N=1`, allocates a flat array of 11,172 slots.
    /// For `N>1`, allocates a single empty branch node (lazy).
    #[inline]
    pub fn new() -> Self {
        let root = if N == 1 {
            Node::new_leaf()
        } else {
            Node::new_branch()
        };
        TreeMap { root, len: 0 }
    }

    /// Returns the number of entries in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the map contains no entries.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the maximum number of entries (`Some`) for N=1, or `None`
    /// for N>1 (tree grows dynamically).
    #[inline]
    pub fn capacity(&self) -> Option<usize> {
        if N == 1 {
            Some(11172)
        } else {
            None
        }
    }
}

impl<const N: usize, V> Default for TreeMap<N, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Core read / write — single Coord (N=1 convenience)
// ---------------------------------------------------------------------------

impl<V> TreeMap<1, V> {
    /// Returns a reference to the value stored at `coord`.
    #[inline]
    pub fn get(&self, coord: Coord) -> Option<&V> {
        self.root.get_value(coord.index() as usize)
    }

    /// Returns a mutable reference to the value stored at `coord`.
    #[inline]
    pub fn get_mut(&mut self, coord: Coord) -> Option<&mut V> {
        self.root.get_value_mut(coord.index() as usize)
    }

    /// Returns `true` if the map contains an entry for `coord`.
    #[inline]
    pub fn contains_key(&self, coord: Coord) -> bool {
        self.get(coord).is_some()
    }

    /// Inserts a value at `coord`, returning the previous value if any.
    #[inline]
    pub fn insert(&mut self, coord: Coord, value: V) -> Option<V> {
        let old = self.root.set_value(coord.index() as usize, value);
        if old.is_none() {
            self.len += 1;
        }
        old
    }

    /// Removes the value at `coord`, returning it if present.
    #[inline]
    pub fn remove(&mut self, coord: Coord) -> Option<V> {
        let old = self.root.take_value(coord.index() as usize);
        if old.is_some() {
            self.len -= 1;
        }
        old
    }

    /// Clears the map, removing all entries.
    pub fn clear(&mut self) {
        // Re-create the root as an empty leaf.
        self.root = Node::new_leaf();
        self.len = 0;
    }
}

// ---------------------------------------------------------------------------
// Core read / write — CoordPath (general N)
// ---------------------------------------------------------------------------

impl<const N: usize, V> TreeMap<N, V> {
    /// Returns a reference to the value stored at `path`.
    pub fn get_path(&self, path: &CoordPath<N>) -> Option<&V> {
        if N == 1 {
            return self.root.get_value(path.coords()[0].index() as usize);
        }
        let mut node = &self.root;
        // Navigate through branch nodes for the first N-1 coordinates.
        for i in 0..(N - 1) {
            let idx = path.coords()[i].index() as usize;
            node = node.get_child(idx)?;
        }
        // Last coordinate indexes into the leaf node.
        let last = path.coords()[N - 1].index() as usize;
        node.get_value(last)
    }

    /// Inserts a value at `path`, returning the previous value if any.
    pub fn insert_path(&mut self, path: &CoordPath<N>, value: V) -> Option<V> {
        if N == 1 {
            let old = self
                .root
                .set_value(path.coords()[0].index() as usize, value);
            if old.is_none() {
                self.len += 1;
            }
            return old;
        }
        let mut node = &mut self.root;
        for i in 0..(N - 1) {
            let idx = path.coords()[i].index() as usize;
            let is_last = i == N - 2;
            node = node.get_child_mut(idx, is_last);
        }
        let last = path.coords()[N - 1].index() as usize;
        let old = node.set_value(last, value);
        if old.is_none() {
            self.len += 1;
        }
        old
    }

    /// Removes the value at `path`, returning it if present.
    pub fn remove_path(&mut self, path: &CoordPath<N>) -> Option<V> {
        if N == 1 {
            let old = self.root.take_value(path.coords()[0].index() as usize);
            if old.is_some() {
                self.len -= 1;
            }
            return old;
        }
        let mut node = &mut self.root;
        for i in 0..(N - 1) {
            let idx = path.coords()[i].index() as usize;
            node = node.get_child_mut_existing(idx)?;
        }
        let last = path.coords()[N - 1].index() as usize;
        let old = node.take_value(last);
        if old.is_some() {
            self.len -= 1;
        }
        old
    }
}

// ---------------------------------------------------------------------------
// Entry API (N=1 only)
// ---------------------------------------------------------------------------

impl<V> TreeMap<1, V> {
    /// Gets the entry for `coord` for in-place manipulation.
    pub fn entry(&mut self, coord: Coord) -> Entry<'_, V> {
        if self.contains_key(coord) {
            Entry::Occupied(OccupiedEntry { map: self, coord })
        } else {
            Entry::Vacant(VacantEntry { map: self, coord })
        }
    }
}

pub enum Entry<'a, V> {
    Occupied(OccupiedEntry<'a, V>),
    Vacant(VacantEntry<'a, V>),
}

pub struct OccupiedEntry<'a, V> {
    map: &'a mut TreeMap<1, V>,
    coord: Coord,
}

impl<'a, V> OccupiedEntry<'a, V> {
    pub fn key(&self) -> Coord {
        self.coord
    }

    pub fn get(&self) -> &V {
        unsafe { self.map.get(self.coord).unwrap_unchecked() }
    }

    pub fn get_mut(&mut self) -> &mut V {
        unsafe { self.map.get_mut(self.coord).unwrap_unchecked() }
    }

    pub fn insert(&mut self, value: V) -> V {
        unsafe { self.map.insert(self.coord, value).unwrap_unchecked() }
    }

    pub fn remove_entry(self) -> V {
        unsafe { self.map.remove(self.coord).unwrap_unchecked() }
    }
}

pub struct VacantEntry<'a, V> {
    map: &'a mut TreeMap<1, V>,
    coord: Coord,
}

impl<'a, V> VacantEntry<'a, V> {
    pub fn key(&self) -> Coord {
        self.coord
    }

    pub fn into_key(self) -> Coord {
        self.coord
    }

    pub fn insert(self, value: V) -> &'a mut V {
        let _ = self.map.insert(self.coord, value);
        unsafe { self.map.get_mut(self.coord).unwrap_unchecked() }
    }
}

impl<'a, V> Entry<'a, V> {
    pub fn key(&self) -> Coord {
        match self {
            Entry::Occupied(e) => e.key(),
            Entry::Vacant(e) => e.key(),
        }
    }

    pub fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert_with(|| default)
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> &'a mut V {
        match self {
            Entry::Occupied(e) => unsafe { e.map.get_mut(e.coord).unwrap_unchecked() },
            Entry::Vacant(e) => e.insert(f()),
        }
    }

    pub fn or_insert_with_key<F: FnOnce(Coord) -> V>(self, f: F) -> &'a mut V {
        match self {
            Entry::Occupied(e) => unsafe { e.map.get_mut(e.coord).unwrap_unchecked() },
            Entry::Vacant(e) => {
                let v = f(e.coord);
                e.insert(v)
            }
        }
    }

    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Self {
        if let Entry::Occupied(ref mut e) = self {
            f(e.get_mut());
        }
        self
    }
}

// ---------------------------------------------------------------------------
// Iteration (N=1 only)
// ---------------------------------------------------------------------------

pub struct Iter<'a, V> {
    node: &'a Node<V>,
    idx: u16,
}

impl<'a, V> Iterator for Iter<'a, V> {
    type Item = (Coord, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < 11172 {
            let coord = Coord::new(self.idx).unwrap();
            self.idx += 1;
            if let Some(val) = self.node.get_value(coord.index() as usize) {
                return Some((coord, val));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(11172 - self.idx as usize))
    }
}

pub struct IterMut<'a, V> {
    node: *mut Node<V>,
    idx: u16,
    _marker: core::marker::PhantomData<&'a mut Node<V>>,
}

impl<'a, V> Iterator for IterMut<'a, V> {
    type Item = (Coord, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= 11172 {
            return None;
        }
        let coord = Coord::new(self.idx).unwrap();
        self.idx += 1;
        // SAFETY: unique mutable access guaranteed by &'a mut Node<V>
        let ptr = unsafe { (*self.node).get_value_mut(coord.index() as usize)? as *mut V };
        let val = unsafe { &mut *ptr };
        Some((coord, val))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(11172 - self.idx as usize))
    }
}

impl<V> TreeMap<1, V> {
    pub fn iter(&self) -> Iter<'_, V> {
        Iter {
            node: &self.root,
            idx: 0,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, V> {
        IterMut {
            node: &mut self.root as *mut Node<V>,
            idx: 0,
            _marker: core::marker::PhantomData,
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = Coord> + '_ {
        self.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.iter().map(|(_, v)| v)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> + '_ {
        self.iter_mut().map(|(_, v)| v)
    }

    pub fn retain<F: FnMut(Coord, &mut V) -> bool>(&mut self, mut f: F) {
        let mut idx = 0u16;
        while idx < 11172 {
            let coord = Coord::new(idx).unwrap();
            idx += 1;
            if let Some(val) = self.root.get_value_mut(coord.index() as usize) {
                if !f(coord, val) {
                    self.root.take_value(coord.index() as usize);
                    self.len -= 1;
                }
            }
        }
    }

    pub fn drain(&mut self) -> Drain<'_, V> {
        Drain { map: self, idx: 0 }
    }
}

pub struct Drain<'a, V> {
    map: &'a mut TreeMap<1, V>,
    idx: u16,
}

impl<'a, V> Iterator for Drain<'a, V> {
    type Item = (Coord, V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < 11172 {
            let coord = Coord::new(self.idx).unwrap();
            self.idx += 1;
            if let Some(val) = self.map.remove(coord) {
                return Some((coord, val));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(11172 - self.idx as usize))
    }
}

impl<'a, V> Drop for Drain<'a, V> {
    fn drop(&mut self) {
        while self.idx < 11172 {
            let coord = Coord::new(self.idx).unwrap();
            self.idx += 1;
            self.map.remove(coord);
        }
    }
}

// ---------------------------------------------------------------------------
// FromIterator (N=1 only)
// ---------------------------------------------------------------------------

impl<V> FromIterator<(Coord, V)> for TreeMap<1, V> {
    fn from_iter<I: IntoIterator<Item = (Coord, V)>>(iter: I) -> Self {
        let mut map = Self::new();
        for (coord, value) in iter {
            map.insert(coord, value);
        }
        map
    }
}

// ---------------------------------------------------------------------------
// IntoIterator (N=1 only) — via a consumed Vec
// ---------------------------------------------------------------------------

impl<V> IntoIterator for TreeMap<1, V> {
    type Item = (Coord, V);
    type IntoIter = alloc::vec::IntoIter<(Coord, V)>;

    fn into_iter(mut self) -> Self::IntoIter {
        let mut vec = alloc::vec::Vec::with_capacity(self.len());
        for i in 0u16..11172 {
            if let Some(val) = self.root.take_value(i as usize) {
                vec.push((Coord::new(i).unwrap(), val));
            }
        }
        vec.into_iter()
    }
}

impl<'a, V> IntoIterator for &'a TreeMap<1, V> {
    type Item = (Coord, &'a V);
    type IntoIter = Iter<'a, V>;

    fn into_iter(self) -> Iter<'a, V> {
        self.iter()
    }
}

impl<'a, V> IntoIterator for &'a mut TreeMap<1, V> {
    type Item = (Coord, &'a mut V);
    type IntoIter = IterMut<'a, V>;

    fn into_iter(self) -> IterMut<'a, V> {
        self.iter_mut()
    }
}

// ---------------------------------------------------------------------------
// Index (N=1 only)
// ---------------------------------------------------------------------------

impl<V> core::ops::Index<Coord> for TreeMap<1, V> {
    type Output = V;

    fn index(&self, coord: Coord) -> &V {
        self.get(coord).expect("TreeMap::index: key not present")
    }
}

impl<V> core::ops::IndexMut<Coord> for TreeMap<1, V> {
    fn index_mut(&mut self, coord: Coord) -> &mut V {
        self.get_mut(coord)
            .expect("TreeMap::index_mut: key not present")
    }
}

// ---------------------------------------------------------------------------
// Type aliases for standard spaces
// ---------------------------------------------------------------------------

/// 1-syllable:  11,172 identifiers (heap-allocated flat array).
/// For no_alloc, use `CoordMap1`.
pub type TreeMap1<V> = TreeMap<1, V>;

/// 2-syllable:  1.25 × 10⁸ identifiers — small KV.
pub type TreeMap2<V> = TreeMap<2, V>;

/// 6-syllable:  1.94 × 10²⁴ identifiers — UUID-scale.
pub type CoordMap6<V> = TreeMap<6, V>;

/// 12-syllable: 2.41 × 10⁶⁷ identifiers — between UUID and SHA-256.
pub type CoordMap12<V> = TreeMap<12, V>;

/// 19-syllable: 1.94 × 10⁷⁷ identifiers — SHA-256-scale (2²⁵⁶).
pub type CoordMap19<V> = TreeMap<19, V>;

// ---------------------------------------------------------------------------
// Tests (inline)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;
    use alloc::string::ToString;

    use alloc::vec::Vec;

    // ── TreeMap<1, _> — flat map tests ──

    #[test]
    fn new_map_is_empty() {
        let map: TreeMap<1, u32> = TreeMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert_eq!(map.capacity(), Some(11172));
    }

    #[test]
    fn insert_and_get() {
        let mut map = TreeMap::<1, u32>::new();
        let c = Coord::new(0).unwrap();
        assert_eq!(map.insert(c, 42), None);
        assert_eq!(map.get(c), Some(&42));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn insert_overwrite() {
        let mut map = TreeMap::<1, u32>::new();
        let c = Coord::new(0).unwrap();
        map.insert(c, 1);
        assert_eq!(map.insert(c, 2), Some(1));
        assert_eq!(map.get(c), Some(&2));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove() {
        let mut map = TreeMap::<1, u32>::new();
        let c = Coord::new(0).unwrap();
        map.insert(c, 42);
        assert_eq!(map.remove(c), Some(42));
        assert_eq!(map.get(c), None);
        assert!(map.is_empty());
    }

    #[test]
    fn contains_key() {
        let mut map = TreeMap::<1, ()>::new();
        let c = Coord::new(0).unwrap();
        assert!(!map.contains_key(c));
        map.insert(c, ());
        assert!(map.contains_key(c));
    }

    #[test]
    fn clear() {
        let mut map = TreeMap::<1, u32>::new();
        map.insert(Coord::new(0).unwrap(), 1);
        map.insert(Coord::new(100).unwrap(), 2);
        map.clear();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn iter_empty() {
        let map: TreeMap<1, u32> = TreeMap::new();
        assert_eq!(map.iter().count(), 0);
    }

    #[test]
    fn iter_non_empty() {
        let mut map = TreeMap::<1, u32>::new();
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
    fn into_iter_consuming() {
        let mut map = TreeMap::<1, &str>::new();
        let c = Coord::new(42).unwrap();
        map.insert(c, "hello");
        let collected: Vec<_> = map.into_iter().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].0, c);
        assert_eq!(collected[0].1, "hello");
    }

    #[test]
    fn from_iterator() {
        let pairs: Vec<_> = (0..5u16)
            .map(|i| (Coord::new(i).unwrap(), i as u64))
            .collect();
        let map: TreeMap<1, u64> = pairs.into_iter().collect();
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn entry_or_insert() {
        let mut map = TreeMap::<1, u32>::new();
        let c = Coord::new(0).unwrap();
        map.entry(c).or_insert(42);
        assert_eq!(map.get(c), Some(&42));
        map.entry(c).or_insert(99);
        assert_eq!(map.get(c), Some(&42));
    }

    #[test]
    fn entry_and_modify() {
        let mut map = TreeMap::<1, u32>::new();
        let c = Coord::new(0).unwrap();
        map.entry(c).and_modify(|v| *v += 1).or_insert(1);
        assert_eq!(map.get(c), Some(&1));
        map.entry(c).and_modify(|v| *v += 1).or_insert(1);
        assert_eq!(map.get(c), Some(&2));
    }

    #[test]
    fn index_trait() {
        let mut map = TreeMap::<1, u32>::new();
        let c = Coord::new(5).unwrap();
        map.insert(c, 42);
        assert_eq!(map[c], 42);
        map[c] = 99;
        assert_eq!(map[c], 99);
    }

    #[test]
    fn default_is_empty() {
        let map: TreeMap<1, u32> = Default::default();
        assert!(map.is_empty());
    }

    // ── TreeMap<1, _> — path API ──

    #[test]
    fn flat_get_path() {
        let mut map = TreeMap::<1, u32>::new();
        let c = Coord::new(42).unwrap();
        map.insert(c, 100);
        assert_eq!(map.get_path(&CoordPath::new([c])), Some(&100));
    }

    #[test]
    fn flat_insert_path() {
        let mut map = TreeMap::<1, u32>::new();
        let c = Coord::new(42).unwrap();
        map.insert_path(&CoordPath::new([c]), 100);
        assert_eq!(map.get(c), Some(&100));
    }

    #[test]
    fn flat_remove_path() {
        let mut map = TreeMap::<1, u32>::new();
        let c = Coord::new(42).unwrap();
        map.insert(c, 100);
        assert_eq!(map.remove_path(&CoordPath::new([c])), Some(100));
        assert!(map.is_empty());
    }

    // ── TreeMap<2, _> — tree map (N=2) ──

    #[test]
    fn tree2_insert_and_get() {
        let mut map = TreeMap::<2, u32>::new();
        let c0 = Coord::new(0).unwrap();
        let c1 = Coord::new(1).unwrap();
        let path = CoordPath::new([c0, c1]);
        assert_eq!(map.insert_path(&path, 42), None);
        assert_eq!(map.get_path(&path), Some(&42));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn tree2_insert_overwrite() {
        let mut map = TreeMap::<2, u32>::new();
        let path = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
        map.insert_path(&path, 1);
        assert_eq!(map.insert_path(&path, 2), Some(1));
        assert_eq!(map.get_path(&path), Some(&2));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn tree2_remove() {
        let mut map = TreeMap::<2, u32>::new();
        let path = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
        map.insert_path(&path, 42);
        assert_eq!(map.remove_path(&path), Some(42));
        assert_eq!(map.get_path(&path), None);
        assert!(map.is_empty());
    }

    #[test]
    fn tree2_independent_paths() {
        let mut map = TreeMap::<2, u32>::new();
        let path_a = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
        let path_b = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
        map.insert_path(&path_a, 10);
        map.insert_path(&path_b, 20);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get_path(&path_a), Some(&10));
        assert_eq!(map.get_path(&path_b), Some(&20));
    }

    #[test]
    fn tree2_remaining_paths_after_remove() {
        let mut map = TreeMap::<2, u32>::new();
        let path_a = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
        let path_b = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
        map.insert_path(&path_a, 10);
        map.insert_path(&path_b, 20);
        map.remove_path(&path_a);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_path(&path_a), None);
        assert_eq!(map.get_path(&path_b), Some(&20));
    }

    // ── TreeMap<6, _> — UUID-scale ──

    #[test]
    fn tree6_basic() {
        let mut map = TreeMap::<6, String>::new();
        let coords = [
            Coord::new(0).unwrap(),
            Coord::new(1).unwrap(),
            Coord::new(2).unwrap(),
            Coord::new(3).unwrap(),
            Coord::new(4).unwrap(),
            Coord::new(5).unwrap(),
        ];
        let path = CoordPath::new(coords);
        map.insert_path(&path, "hello".to_string());
        assert_eq!(map.get_path(&path).map(|s| s.as_str()), Some("hello"));
    }

    #[test]
    fn tree6_missing_path() {
        let map = TreeMap::<6, u32>::new();
        let path = CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]);
        assert_eq!(map.get_path(&path), None);
    }

    // ── Type aliases ──

    #[test]
    fn type_aliases_exist() {
        let _m1: TreeMap1<u32> = TreeMap::new();
        let _m2: TreeMap2<u32> = TreeMap::new();
        let _m6: CoordMap6<u32> = TreeMap::new();
        let _m12: CoordMap12<u32> = TreeMap::new();
        let _m19: CoordMap19<u32> = TreeMap::new();
    }

    #[test]
    fn coord_map1_is_coord_map_1() {
        let mut m1: TreeMap1<u32> = TreeMap::new();
        let c = Coord::new(0).unwrap();
        m1.insert(c, 42);
        assert_eq!(m1.get(c), Some(&42));
    }

    #[test]
    fn coord_map6_uuid_scale() {
        let mut map: CoordMap6<u32> = TreeMap::new();
        let path = CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]);
        map.insert_path(&path, 42);
        assert_eq!(map.get_path(&path), Some(&42));
    }

    #[test]
    fn max_depth_insert() {
        let mut map = TreeMap::<19, u32>::new();
        let coords = [
            Coord::new(0).unwrap(),
            Coord::new(1).unwrap(),
            Coord::new(2).unwrap(),
            Coord::new(3).unwrap(),
            Coord::new(4).unwrap(),
            Coord::new(5).unwrap(),
            Coord::new(6).unwrap(),
            Coord::new(7).unwrap(),
            Coord::new(8).unwrap(),
            Coord::new(9).unwrap(),
            Coord::new(10).unwrap(),
            Coord::new(11).unwrap(),
            Coord::new(12).unwrap(),
            Coord::new(13).unwrap(),
            Coord::new(14).unwrap(),
            Coord::new(15).unwrap(),
            Coord::new(16).unwrap(),
            Coord::new(17).unwrap(),
            Coord::new(18).unwrap(),
        ];
        let path = CoordPath::new(coords);
        map.insert_path(&path, 42);
        assert_eq!(map.get_path(&path), Some(&42));
        assert_eq!(map.len(), 1);
    }
}

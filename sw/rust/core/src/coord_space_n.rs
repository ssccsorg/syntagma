use crate::coord::Coord;
use crate::coord_path::CoordPath;
use alloc::boxed::Box;
use alloc::vec::Vec;

// ---------------------------------------------------------------------------
// CoordSpace: hash-less, collision-free N-level address table (N>1)
// ---------------------------------------------------------------------------

/// A hash-less, collision-free, N-level address table indexed by [`CoordPath`]
/// for N > 1. Requires a heap allocator.
///
/// For single-syllable addressing without heap allocation, use [`CoordSpace`].
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
#[derive(Clone)]
pub struct CoordSpaceN<const N: usize, V> {
    root: Node<V>,
    len: usize,
}

/// A single node in the tree — zero unsafe code.
///
/// - `Leaf`: boxed slice of 11,172 `Option<V>` slots.
/// - `Branch`: boxed slice of 11,172 `Option<Box<Node<V>>>` slots.
#[derive(Clone)]
enum Node<V> {
    Leaf(Box<[Option<V>]>),
    Branch(Box<[Option<Box<Node<V>>>]>),
}

impl<V> Node<V> {
    #[inline]
    fn new_leaf() -> Self {
        Node::Leaf(
            (0..11172)
                .map(|_| None)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }

    #[inline]
    fn new_branch() -> Self {
        Node::Branch(
            (0..11172)
                .map(|_| None)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
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

impl<const N: usize, V> CoordSpaceN<N, V> {
    /// Creates an empty `CoordSpace`.
    ///
    /// For `N=1`, allocates a flat array of 11,172 slots.
    /// For `N>1`, allocates a single empty branch node (lazy).
    ///
    /// # Panics
    ///
    /// Panics if `N` is 0 (depth must be at least 1).
    #[inline]
    pub fn new() -> Self {
        assert!(N > 0, "CoordSpace depth N must be at least 1");
        let root = if N == 1 {
            Node::new_leaf()
        } else {
            Node::new_branch()
        };
        CoordSpaceN { root, len: 0 }
    }

    /// Returns the number of entries in the space.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the space contains no entries.
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

impl<const N: usize, V> Default for CoordSpaceN<N, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Debug, PartialEq — manual impls for compact output and value comparison
// ---------------------------------------------------------------------------

impl<V: core::fmt::Debug> core::fmt::Debug for Node<V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Show occupied slot count instead of dumping 11,172 entries.
        match self {
            Node::Leaf(slots) => {
                let occupied = slots.iter().filter(|s| s.is_some()).count();
                f.debug_struct("Leaf").field("occupied", &occupied).finish()
            }
            Node::Branch(children) => {
                let occupied = children.iter().filter(|c| c.is_some()).count();
                f.debug_struct("Branch")
                    .field("children", &occupied)
                    .finish()
            }
        }
    }
}

impl<const N: usize, V: core::fmt::Debug> core::fmt::Debug for CoordSpaceN<N, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CoordSpace")
            .field("N", &N)
            .field("len", &self.len)
            .field("root", &self.root)
            .finish()
    }
}

impl<V: PartialEq> PartialEq for Node<V> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::Leaf(a), Node::Leaf(b)) => a == b,
            (Node::Branch(a), Node::Branch(b)) => a == b,
            _ => false,
        }
    }
}

impl<const N: usize, V: PartialEq> PartialEq for CoordSpaceN<N, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.root == other.root
    }
}

impl<const N: usize, V: PartialEq> Eq for CoordSpaceN<N, V> {}

// ---------------------------------------------------------------------------
// Core read / write — single Coord (N=1 convenience)
// ---------------------------------------------------------------------------

impl<V> CoordSpaceN<1, V> {
    /// Returns a reference to the value stored at `coord`.
    #[inline]
    pub fn at(&self, coord: &Coord) -> Option<&V> {
        self.root.get_value(coord.index() as usize)
    }

    /// Returns a mutable reference to the value stored at `coord`.
    #[inline]
    pub fn at_mut(&mut self, coord: &Coord) -> Option<&mut V> {
        self.root.get_value_mut(coord.index() as usize)
    }

    /// Returns `true` if the space contains an entry for `coord`.
    #[inline]
    pub fn occupied(&self, coord: &Coord) -> bool {
        self.at(coord).is_some()
    }

    /// Inserts a value at `coord`, returning the previous value if any.
    #[inline]
    pub fn place(&mut self, coord: Coord, value: V) -> Option<V> {
        let old = self.root.set_value(coord.index() as usize, value);
        if old.is_none() {
            self.len += 1;
        }
        old
    }

    /// Removes the value at `coord`, returning it if present.
    #[inline]
    pub fn vacate(&mut self, coord: &Coord) -> Option<V> {
        let old = self.root.take_value(coord.index() as usize);
        if old.is_some() {
            self.len -= 1;
        }
        old
    }
}

// ---------------------------------------------------------------------------
// Core read / write — CoordPath (general N)
// ---------------------------------------------------------------------------

impl<const N: usize, V> CoordSpaceN<N, V> {
    /// Returns a reference to the value stored at `path`.
    pub fn at_path(&self, path: &CoordPath<N>) -> Option<&V> {
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

    /// Returns a mutable reference to the value stored at `path`.
    pub fn at_path_mut(&mut self, path: &CoordPath<N>) -> Option<&mut V> {
        if N == 1 {
            return self.root.get_value_mut(path.coords()[0].index() as usize);
        }
        let mut node = &mut self.root;
        for i in 0..(N - 1) {
            let idx = path.coords()[i].index() as usize;
            node = node.get_child_mut_existing(idx)?;
        }
        let last = path.coords()[N - 1].index() as usize;
        node.get_value_mut(last)
    }

    /// Inserts a value at `path`, returning the previous value if any.
    pub fn place_path(&mut self, path: &CoordPath<N>, value: V) -> Option<V> {
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
    pub fn vacate_path(&mut self, path: &CoordPath<N>) -> Option<V> {
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

    /// Removes all entries, preserving allocated tree structure.
    /// O(entries) — walks the tree clearing occupied slots.
    pub fn clear(&mut self) {
        clear_node(&mut self.root);
        self.len = 0;
    }
}

fn clear_node<V>(node: &mut Node<V>) {
    match node {
        Node::Leaf(slots) => {
            for slot in slots.iter_mut() {
                *slot = None;
            }
        }
        Node::Branch(children) => {
            for child in children.iter_mut().flatten() {
                clear_node(child);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Entry API (N=1 only)
// ---------------------------------------------------------------------------

impl<V> CoordSpaceN<1, V> {
    /// Gets the entry for `coord` for in-place manipulation.
    pub fn entry(&mut self, coord: Coord) -> Entry<'_, V> {
        if self.occupied(&coord) {
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
    map: &'a mut CoordSpaceN<1, V>,
    coord: Coord,
}

impl<'a, V> OccupiedEntry<'a, V> {
    pub fn coord(&self) -> Coord {
        self.coord
    }

    pub fn at(&self) -> &V {
        unsafe { self.map.at(&self.coord).unwrap_unchecked() }
    }

    pub fn at_mut(&mut self) -> &mut V {
        unsafe { self.map.at_mut(&self.coord).unwrap_unchecked() }
    }

    pub fn place(&mut self, value: V) -> V {
        unsafe { self.map.place(self.coord, value).unwrap_unchecked() }
    }

    pub fn remove_entry(self) -> V {
        unsafe { self.map.vacate(&self.coord).unwrap_unchecked() }
    }
}

pub struct VacantEntry<'a, V> {
    map: &'a mut CoordSpaceN<1, V>,
    coord: Coord,
}

impl<'a, V> VacantEntry<'a, V> {
    pub fn coord(&self) -> Coord {
        self.coord
    }

    pub fn into_key(self) -> Coord {
        self.coord
    }

    pub fn place(self, value: V) -> &'a mut V {
        let _ = self.map.place(self.coord, value);
        unsafe { self.map.at_mut(&self.coord).unwrap_unchecked() }
    }
}

impl<'a, V> Entry<'a, V> {
    pub fn coord(&self) -> Coord {
        match self {
            Entry::Occupied(e) => e.coord(),
            Entry::Vacant(e) => e.coord(),
        }
    }

    pub fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert_with(|| default)
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> &'a mut V {
        match self {
            Entry::Occupied(e) => unsafe { e.map.at_mut(&e.coord).unwrap_unchecked() },
            Entry::Vacant(e) => e.place(f()),
        }
    }

    pub fn or_insert_with_key<F: FnOnce(Coord) -> V>(self, f: F) -> &'a mut V {
        match self {
            Entry::Occupied(e) => unsafe { e.map.at_mut(&e.coord).unwrap_unchecked() },
            Entry::Vacant(e) => {
                let v = f(e.coord);
                e.place(v)
            }
        }
    }

    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Self {
        if let Entry::Occupied(ref mut e) = self {
            f(e.at_mut());
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

// ---------------------------------------------------------------------------
// TreeIter — lazy stack-based DFS iterator (no pre-collection)
// ---------------------------------------------------------------------------

/// Pre-collected iterator over a CoordSpaceN tree.
///
/// Collects all `(indices, &V)` pairs upfront via a single tree walk,
/// then returns them on `next()`. The upfront walk is O(entries × depth)
/// with no redundant slot scans — each node's occupied slots are found
/// during the single walk and their `&V` references are stored directly.
pub struct TreeIter<'a, const N: usize, V> {
    entries: alloc::vec::IntoIter<(CoordPath<N>, &'a V)>,
}

impl<'a, const N: usize, V> Iterator for TreeIter<'a, N, V> {
    type Item = (CoordPath<N>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.entries.size_hint()
    }
}

/// Recursively collect all `(path, &V)` pairs from a node at the given depth.
fn collect_entries<'a, const M: usize, V>(
    node: &'a Node<V>,
    depth: usize,
    current: &mut [u16; M],
    out: &mut Vec<(CoordPath<M>, &'a V)>,
) {
    match node {
        Node::Leaf(slots) => {
            for (i, slot) in slots.iter().enumerate() {
                if let Some(val) = slot.as_ref() {
                    current[depth] = i as u16;
                    let coords = core::array::from_fn(|j| Coord::new(current[j]).unwrap());
                    out.push((CoordPath::new(coords), val));
                }
            }
        }
        Node::Branch(children) => {
            for (i, child) in children.iter().enumerate() {
                if let Some(child) = child {
                    current[depth] = i as u16;
                    collect_entries::<M, V>(child, depth + 1, current, out);
                }
            }
        }
    }
}

impl<const N: usize, V> CoordSpaceN<N, V> {
    /// Returns an iterator over all `(path, value)` pairs in the tree.
    /// Collects entries upfront via a single tree walk — O(entries × depth).
    /// For N=1, consider using `iter_flat()` instead for `(Coord, &V)` items.
    pub fn iter_tree(&self) -> TreeIter<'_, N, V> {
        let mut entries = Vec::new();
        let mut current = [0u16; N];
        collect_entries::<N, V>(&self.root, 0, &mut current, &mut entries);
        TreeIter {
            entries: entries.into_iter(),
        }
    }

    /// Returns an iterator over entries whose path starts with the given prefix.
    ///
    /// Only traverses the subtree under `prefix`, skipping all other branches.
    /// Returns `None` if the prefix depth exceeds N or if the prefix path does
    /// not exist in the tree.
    ///
    /// # Example
    ///
    /// ```text
    /// // See the #[test] fn iter_prefix_test below for a runnable example.
    /// ```
    pub fn iter_prefix(&self, prefix: &[Coord]) -> Option<TreeIter<'_, N, V>> {
        let k = prefix.len();
        if k >= N {
            return None;
        }
        // Navigate to the branch at depth k.
        let mut node = &self.root;
        let mut current = [0u16; N];
        for (i, coord) in prefix.iter().enumerate() {
            current[i] = coord.index();
            match node {
                Node::Branch(children) => {
                    node = children[coord.index() as usize].as_ref()?;
                }
                Node::Leaf(_) => return None,
            }
        }
        let mut entries = Vec::new();
        collect_entries::<N, V>(node, k, &mut current, &mut entries);
        Some(TreeIter {
            entries: entries.into_iter(),
        })
    }
}

// ---------------------------------------------------------------------------
// Existing 1-syllable iteration
// ---------------------------------------------------------------------------

impl<V> CoordSpaceN<1, V> {
    pub fn iter_flat(&self) -> Iter<'_, V> {
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

    pub fn coords(&self) -> impl Iterator<Item = Coord> + '_ {
        self.iter_flat().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.iter_flat().map(|(_, v)| v)
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
    map: &'a mut CoordSpaceN<1, V>,
    idx: u16,
}

impl<'a, V> Iterator for Drain<'a, V> {
    type Item = (Coord, V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < 11172 {
            let coord = Coord::new(self.idx).unwrap();
            self.idx += 1;
            if let Some(val) = self.map.vacate(&coord) {
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
            self.map.vacate(&coord);
        }
    }
}

// ---------------------------------------------------------------------------
// FromIterator (N=1 only)
// ---------------------------------------------------------------------------

impl<V> FromIterator<(Coord, V)> for CoordSpaceN<1, V> {
    fn from_iter<I: IntoIterator<Item = (Coord, V)>>(iter: I) -> Self {
        let mut map = Self::new();
        for (coord, value) in iter {
            map.place(coord, value);
        }
        map
    }
}

// ---------------------------------------------------------------------------
// IntoIterator (N=1 only) — via a consumed Vec
// ---------------------------------------------------------------------------

impl<V> IntoIterator for CoordSpaceN<1, V> {
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

impl<'a, V> IntoIterator for &'a CoordSpaceN<1, V> {
    type Item = (Coord, &'a V);
    type IntoIter = Iter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_flat()
    }
}

impl<'a, V> IntoIterator for &'a mut CoordSpaceN<1, V> {
    type Item = (Coord, &'a mut V);
    type IntoIter = IterMut<'a, V>;

    fn into_iter(self) -> IterMut<'a, V> {
        self.iter_mut()
    }
}

// ---------------------------------------------------------------------------
// Index (N=1 only)
// ---------------------------------------------------------------------------

impl<V> core::ops::Index<Coord> for CoordSpaceN<1, V> {
    type Output = V;

    fn index(&self, coord: Coord) -> &V {
        self.at(&coord)
            .expect("CoordSpaceN::index: key not present")
    }
}

impl<V> core::ops::IndexMut<Coord> for CoordSpaceN<1, V> {
    fn index_mut(&mut self, coord: Coord) -> &mut V {
        self.at_mut(&coord)
            .expect("CoordSpaceN::index_mut: key not present")
    }
}

// ---------------------------------------------------------------------------
// Type aliases for standard spaces
// ---------------------------------------------------------------------------

/// 1-syllable:  11,172 identifiers (heap-allocated flat array).
/// For no_alloc, use `CoordSpace1`.
pub type CoordSpace1<V> = CoordSpaceN<1, V>;

/// 2-syllable:  1.25 × 10⁸ identifiers — small KV.
pub type CoordSpace2<V> = CoordSpaceN<2, V>;

/// 3-syllable:  1.39 × 10¹² identifiers — medium KV.
pub type CoordSpace3<V> = CoordSpaceN<3, V>;

/// 6-syllable:  1.94 × 10²⁴ identifiers — UUID-scale.
pub type CoordSpace6<V> = CoordSpaceN<6, V>;

/// 12-syllable: 2.41 × 10⁶⁷ identifiers — between UUID and SHA-256.
pub type CoordSpace12<V> = CoordSpaceN<12, V>;

/// 19-syllable: 1.94 × 10⁷⁷ identifiers — SHA-256-scale (2²⁵⁶).
pub type CoordSpace19<V> = CoordSpaceN<19, V>;

// ---------------------------------------------------------------------------
// Tests (inline)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;
    use alloc::string::ToString;

    use alloc::vec::Vec;

    // ── CoordSpaceN<1, _> — flat map tests ──

    #[test]
    fn new_map_is_empty() {
        let map: CoordSpaceN<1, u32> = CoordSpaceN::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert_eq!(map.capacity(), Some(11172));
    }

    #[test]
    fn insert_and_get() {
        let mut map = CoordSpaceN::<1, u32>::new();
        let c = Coord::new(0).unwrap();
        assert_eq!(map.place(c, 42), None);
        assert_eq!(map.at(&c), Some(&42));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn insert_overwrite() {
        let mut map = CoordSpaceN::<1, u32>::new();
        let c = Coord::new(0).unwrap();
        map.place(c, 1);
        assert_eq!(map.place(c, 2), Some(1));
        assert_eq!(map.at(&c), Some(&2));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn vacate() {
        let mut map = CoordSpaceN::<1, u32>::new();
        let c = Coord::new(0).unwrap();
        map.place(c, 42);
        assert_eq!(map.vacate(&c), Some(42));
        assert_eq!(map.at(&c), None);
        assert!(map.is_empty());
    }

    #[test]
    fn contains_key() {
        let mut map = CoordSpaceN::<1, ()>::new();
        let c = Coord::new(0).unwrap();
        assert!(!map.occupied(&c));
        map.place(c, ());
        assert!(map.occupied(&c));
    }

    #[test]
    fn clear() {
        let mut map = CoordSpaceN::<1, u32>::new();
        map.place(Coord::new(0).unwrap(), 1);
        map.place(Coord::new(100).unwrap(), 2);
        map.clear();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn iter_empty() {
        let map: CoordSpaceN<1, u32> = CoordSpaceN::new();
        assert_eq!(map.iter_flat().count(), 0);
    }

    #[test]
    fn iter_non_empty() {
        let mut map = CoordSpaceN::<1, u32>::new();
        let c1 = Coord::new(0).unwrap();
        let c2 = Coord::new(9999).unwrap();
        map.place(c1, 10);
        map.place(c2, 20);
        let entries: Vec<_> = map.iter_flat().collect();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&(c1, &10)));
        assert!(entries.contains(&(c2, &20)));
    }

    #[test]
    fn into_iter_consuming() {
        let mut map = CoordSpaceN::<1, &str>::new();
        let c = Coord::new(42).unwrap();
        map.place(c, "hello");
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
        let map: CoordSpaceN<1, u64> = pairs.into_iter().collect();
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn entry_or_insert() {
        let mut map = CoordSpaceN::<1, u32>::new();
        let c = Coord::new(0).unwrap();
        map.entry(c).or_insert(42);
        assert_eq!(map.at(&c), Some(&42));
        map.entry(c).or_insert(99);
        assert_eq!(map.at(&c), Some(&42));
    }

    #[test]
    fn entry_and_modify() {
        let mut map = CoordSpaceN::<1, u32>::new();
        let c = Coord::new(0).unwrap();
        map.entry(c).and_modify(|v| *v += 1).or_insert(1);
        assert_eq!(map.at(&c), Some(&1));
        map.entry(c).and_modify(|v| *v += 1).or_insert(1);
        assert_eq!(map.at(&c), Some(&2));
    }

    #[test]
    fn index_trait() {
        let mut map = CoordSpaceN::<1, u32>::new();
        let c = Coord::new(5).unwrap();
        map.place(c, 42);
        assert_eq!(map[c], 42);
        map[c] = 99;
        assert_eq!(map[c], 99);
    }

    #[test]
    fn default_is_empty() {
        let map: CoordSpaceN<1, u32> = Default::default();
        assert!(map.is_empty());
    }

    // ── CoordSpaceN<1, _> — path API ──

    #[test]
    fn flat_get_path() {
        let mut map = CoordSpaceN::<1, u32>::new();
        let c = Coord::new(42).unwrap();
        map.place(c, 100);
        assert_eq!(map.at_path(&CoordPath::new([c])), Some(&100));
    }

    #[test]
    fn flat_insert_path() {
        let mut map = CoordSpaceN::<1, u32>::new();
        let c = Coord::new(42).unwrap();
        map.place_path(&CoordPath::new([c]), 100);
        assert_eq!(map.at(&c), Some(&100));
    }

    #[test]
    fn flat_remove_path() {
        let mut map = CoordSpaceN::<1, u32>::new();
        let c = Coord::new(42).unwrap();
        map.place(c, 100);
        assert_eq!(map.vacate_path(&CoordPath::new([c])), Some(100));
        assert!(map.is_empty());
    }

    // ── CoordSpaceN<2, _> — tree map (N=2) ──

    #[test]
    fn tree2_insert_and_get() {
        let mut map = CoordSpaceN::<2, u32>::new();
        let c0 = Coord::new(0).unwrap();
        let c1 = Coord::new(1).unwrap();
        let path = CoordPath::new([c0, c1]);
        assert_eq!(map.place_path(&path, 42), None);
        assert_eq!(map.at_path(&path), Some(&42));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn tree2_insert_overwrite() {
        let mut map = CoordSpaceN::<2, u32>::new();
        let path = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
        map.place_path(&path, 1);
        assert_eq!(map.place_path(&path, 2), Some(1));
        assert_eq!(map.at_path(&path), Some(&2));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn tree2_remove() {
        let mut map = CoordSpaceN::<2, u32>::new();
        let path = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
        map.place_path(&path, 42);
        assert_eq!(map.vacate_path(&path), Some(42));
        assert_eq!(map.at_path(&path), None);
        assert!(map.is_empty());
    }

    #[test]
    fn tree2_independent_paths() {
        let mut map = CoordSpaceN::<2, u32>::new();
        let path_a = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
        let path_b = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
        map.place_path(&path_a, 10);
        map.place_path(&path_b, 20);
        assert_eq!(map.len(), 2);
        assert_eq!(map.at_path(&path_a), Some(&10));
        assert_eq!(map.at_path(&path_b), Some(&20));
    }

    #[test]
    fn tree2_remaining_paths_after_remove() {
        let mut map = CoordSpaceN::<2, u32>::new();
        let path_a = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
        let path_b = CoordPath::new([Coord::new(0).unwrap(), Coord::new(1).unwrap()]);
        map.place_path(&path_a, 10);
        map.place_path(&path_b, 20);
        map.vacate_path(&path_a);
        assert_eq!(map.len(), 1);
        assert_eq!(map.at_path(&path_a), None);
        assert_eq!(map.at_path(&path_b), Some(&20));
    }

    // ── CoordSpaceN<6, _> — UUID-scale ──

    #[test]
    fn tree6_basic() {
        let mut map = CoordSpaceN::<6, String>::new();
        let coords = [
            Coord::new(0).unwrap(),
            Coord::new(1).unwrap(),
            Coord::new(2).unwrap(),
            Coord::new(3).unwrap(),
            Coord::new(4).unwrap(),
            Coord::new(5).unwrap(),
        ];
        let path = CoordPath::new(coords);
        map.place_path(&path, "hello".to_string());
        assert_eq!(map.at_path(&path).map(|s| s.as_str()), Some("hello"));
    }

    #[test]
    fn tree6_missing_path() {
        let map = CoordSpaceN::<6, u32>::new();
        let path = CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]);
        assert_eq!(map.at_path(&path), None);
    }

    // ── Type aliases ──

    #[test]
    fn type_aliases_exist() {
        let _m1: CoordSpace1<u32> = CoordSpaceN::new();
        let _m2: CoordSpace2<u32> = CoordSpaceN::new();
        let _m6: CoordSpace6<u32> = CoordSpaceN::new();
        let _m12: CoordSpace12<u32> = CoordSpaceN::new();
        let _m19: CoordSpace19<u32> = CoordSpaceN::new();
    }

    #[test]
    fn coord_space12_basic() {
        let mut space: CoordSpace12<String> = CoordSpaceN::new();
        let path = CoordPath::new(core::array::from_fn(|i| Coord::new(i as u16).unwrap()));
        space.place_path(&path, "hello".to_string());
        assert_eq!(space.at_path(&path).map(|s| s.as_str()), Some("hello"));
        assert_eq!(space.len(), 1);
        assert_eq!(space.vacate_path(&path), Some("hello".to_string()));
        assert!(space.is_empty());
    }

    // ── Clear for N>1 ──

    #[test]
    fn tree2_clear() {
        let mut map = CoordSpaceN::<2, u32>::new();
        map.place_path(
            &CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]),
            1,
        );
        map.place_path(
            &CoordPath::new([Coord::new(1).unwrap(), Coord::new(1).unwrap()]),
            2,
        );
        assert_eq!(map.len(), 2);
        map.clear();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        // Reuse after clear
        map.place_path(
            &CoordPath::new([Coord::new(2).unwrap(), Coord::new(2).unwrap()]),
            3,
        );
        assert_eq!(
            map.at_path(&CoordPath::new([
                Coord::new(2).unwrap(),
                Coord::new(2).unwrap()
            ])),
            Some(&3)
        );
    }

    #[test]
    fn tree6_clear() {
        let mut map = CoordSpaceN::<6, u32>::new();
        let path = CoordPath::new(core::array::from_fn(|i| Coord::new(i as u16).unwrap()));
        map.place_path(&path, 42);
        assert_eq!(map.len(), 1);
        map.clear();
        assert!(map.is_empty());
    }

    // ── Clone / PartialEq / Debug ──

    #[test]
    fn tree2_clone_independent() {
        let mut a = CoordSpaceN::<2, u32>::new();
        a.place_path(
            &CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]),
            1,
        );
        let mut b = a.clone();
        b.place_path(
            &CoordPath::new([Coord::new(1).unwrap(), Coord::new(1).unwrap()]),
            2,
        );
        assert_eq!(a.len(), 1);
        assert_eq!(b.len(), 2);
        assert_eq!(
            a.at_path(&CoordPath::new([
                Coord::new(0).unwrap(),
                Coord::new(0).unwrap()
            ])),
            Some(&1)
        );
        assert_eq!(
            b.at_path(&CoordPath::new([
                Coord::new(0).unwrap(),
                Coord::new(0).unwrap()
            ])),
            Some(&1)
        );
        assert_eq!(
            b.at_path(&CoordPath::new([
                Coord::new(1).unwrap(),
                Coord::new(1).unwrap()
            ])),
            Some(&2)
        );
    }

    #[test]
    fn tree2_partial_eq() {
        let mut a = CoordSpaceN::<2, u32>::new();
        let mut b = CoordSpaceN::<2, u32>::new();
        let p = CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]);
        a.place_path(&p, 42);
        b.place_path(&p, 42);
        assert_eq!(a, b);
        b.place_path(
            &CoordPath::new([Coord::new(1).unwrap(), Coord::new(1).unwrap()]),
            99,
        );
        assert_ne!(a, b);
    }

    #[test]
    fn tree2_debug_format() {
        let mut map = CoordSpaceN::<2, u32>::new();
        map.place_path(
            &CoordPath::new([Coord::new(0).unwrap(), Coord::new(0).unwrap()]),
            1,
        );
        let s = alloc::format!("{:?}", map);
        assert!(s.contains("CoordSpace"));
        assert!(s.contains("N: 2"));
        assert!(s.contains("len: 1"));
    }

    #[test]
    fn coord_space1_is_coord_space_1() {
        let mut s1: CoordSpace1<u32> = CoordSpaceN::new();
        let c = Coord::new(0).unwrap();
        s1.place(c, 42);
        assert_eq!(s1.at(&c), Some(&42));
    }

    #[test]
    fn coord_space6_uuid_scale() {
        let mut space: CoordSpace6<u32> = CoordSpaceN::new();
        let path = CoordPath::new([
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
            Coord::new(0).unwrap(),
        ]);
        space.place_path(&path, 42);
        assert_eq!(space.at_path(&path), Some(&42));
    }

    #[test]
    fn max_depth_insert() {
        let mut map = CoordSpaceN::<19, u32>::new();
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
        map.place_path(&path, 42);
        assert_eq!(map.at_path(&path), Some(&42));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn iter_prefix_test() {
        let mut space: CoordSpaceN<2, u32> = CoordSpaceN::new();
        space.place_path(
            &CoordPath::new([Coord::new(42).unwrap(), Coord::new(1).unwrap()]),
            10,
        );
        space.place_path(
            &CoordPath::new([Coord::new(42).unwrap(), Coord::new(2).unwrap()]),
            20,
        );
        space.place_path(
            &CoordPath::new([Coord::new(99).unwrap(), Coord::new(0).unwrap()]),
            30,
        );
        let prefix = [Coord::new(42).unwrap()];
        let mut results: Vec<_> = space.iter_prefix(&prefix).unwrap().collect();
        results.sort_by_key(|(_, v)| *v);
        assert_eq!(results.len(), 2);
        assert_eq!(*results[0].1, 10);
        assert_eq!(*results[1].1, 20);
        let missing = [Coord::new(11111).unwrap()];
        assert!(space.iter_prefix(&missing).is_none());
    }
}

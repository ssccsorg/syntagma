use crate::coord::Coord;
use crate::coord_path::CoordPath;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

// ---------------------------------------------------------------------------
// CoordSpace: hash-less, collision-free N-level address table (N>1)
// ---------------------------------------------------------------------------

/// A hash-less, collision-free, N-level address table indexed by [`CoordPath`]
/// for N > 1. Requires a heap allocator.
///
/// For single-character addressing without heap allocation, use [`CoordSpace`].
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

/// A single frame on the DFS stack. Holds a node reference and the next
/// slot index to scan. Stack depth is at most N (tree depth), guaranteeing
/// minimal allocation regardless of entry count.
struct StackFrame<'a, V> {
    node: &'a Node<V>,
    idx: usize,
}

/// Lazy stack-based DFS iterator over a CoordSpaceN tree.
///
/// Uses a small stack (at most N frames) instead of pre-collecting all
/// entries into a Vec. Each `next()` advances the DFS in place, yielding
/// entries one at a time with O(1) amortized cost per element.
pub struct TreeIter<'a, const N: usize, V> {
    stack: Vec<StackFrame<'a, V>>,
    path: [u16; N],
    base_depth: usize,
}

impl<'a, const N: usize, V> Iterator for TreeIter<'a, N, V> {
    type Item = (CoordPath<N>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            let stack_len = self.stack.len();
            if stack_len == 0 {
                return None;
            }
            let depth = self.base_depth + stack_len - 1;
            // SAFETY: stack is non-empty (checked above via stack_len > 0).
            let frame = unsafe { self.stack.last_mut().unwrap_unchecked() };
            match frame.node {
                Node::Leaf(slots) => {
                    // Scan remaining leaf slots for an occupied one.
                    for i in frame.idx..slots.len() {
                        if slots[i].is_some() {
                            frame.idx = i + 1;
                            self.path[depth] = i as u16;
                            let coords = core::array::from_fn(|j| {
                                // SAFETY: self.path[j] is always < N_VALID (11172)
                                // because it's set from slot indices which are bounded
                                // by the tree's fixed 11172-slot arrays.
                                unsafe { Coord::new_unchecked(self.path[j]) }
                            });
                            return Some((
                                CoordPath::new(coords),
                                // SAFETY: checked is_some() in the enclosing if
                                unsafe { slots[i].as_ref().unwrap_unchecked() },
                            ));
                        }
                    }
                    // Leaf exhausted; backtrack to parent.
                    self.stack.pop();
                }
                Node::Branch(children) => {
                    // Scan remaining branch slots for an occupied child.
                    for i in frame.idx..children.len() {
                        if let Some(child) = &children[i] {
                            frame.idx = i + 1;
                            self.path[depth] = i as u16;
                            self.stack.push(StackFrame {
                                node: child,
                                idx: 0,
                            });
                            continue 'outer;
                        }
                    }
                    // Branch exhausted; backtrack to parent.
                    self.stack.pop();
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Cannot determine remaining count without traversing.
        (0, None)
    }
}

impl<const N: usize, V> CoordSpaceN<N, V> {
    /// Returns an iterator over all `(path, value)` pairs in the tree.
    ///
    /// Uses a lazy stack-based DFS with O(1) amortized `next()` and no
    /// pre-collection allocation. Stack depth is bounded by N.
    /// For N=1, consider using `iter_flat()` instead for `(Coord, &V)` items.
    pub fn iter_tree(&self) -> TreeIter<'_, N, V> {
        TreeIter {
            stack: vec![StackFrame {
                node: &self.root,
                idx: 0,
            }],
            path: [0u16; N],
            base_depth: 0,
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
        // Navigate to the subtree at depth k.
        let mut node = &self.root;
        let mut path = [0u16; N];
        for (i, coord) in prefix.iter().enumerate() {
            path[i] = coord.index();
            match node {
                Node::Branch(children) => {
                    node = children[coord.index() as usize].as_ref()?;
                }
                Node::Leaf(_) => return None,
            }
        }
        Some(TreeIter {
            stack: vec![StackFrame { node, idx: 0 }],
            path,
            base_depth: k,
        })
    }
}

// ---------------------------------------------------------------------------
// Existing 1-character iteration
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

/// 1-character:  11,172 identifiers (heap-allocated flat array).
/// For no_alloc (dense zeroed array), use `CoordSpace`.
pub type CoordSpaceN1<V> = CoordSpaceN<1, V>;

/// 2-character:  1.25 × 10⁸ identifiers — small KV.
pub type CoordSpaceN2<V> = CoordSpaceN<2, V>;

/// 3-character:  1.39 × 10¹² identifiers — medium KV.
pub type CoordSpaceN3<V> = CoordSpaceN<3, V>;

/// 6-character:  1.94 × 10²⁴ identifiers — UUID-scale.
pub type CoordSpaceN6<V> = CoordSpaceN<6, V>;

/// 12-character: 2.41 × 10⁶⁷ identifiers — between UUID and SHA-256.
pub type CoordSpaceN12<V> = CoordSpaceN<12, V>;

/// 19-character: 1.94 × 10⁷⁷ identifiers — SHA-256-scale (2²⁵⁶).
pub type CoordSpaceN19<V> = CoordSpaceN<19, V>;

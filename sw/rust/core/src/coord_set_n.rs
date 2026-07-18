use crate::coord_path::CoordPath;
use crate::coord_space_n::{CoordSpaceN, TreeIter};

// ---------------------------------------------------------------------------
// CoordSetN — sparse N-dimensional coordinate set
// ---------------------------------------------------------------------------

/// A sparse set of coordinates of depth N (>1), backed by a [`CoordSpaceN`]
/// tree. Memory is allocated lazily: only coordinates that are actually
/// inserted consume nodes.
///
/// For N=1, use [`CoordSet`](crate::coord_set::CoordSet) instead — it is a
/// dense bitset with zero allocation, bitwise operations, and approximately
/// 10x faster single-operation throughput.
///
/// # Set operations
///
/// `union`, `intersection`, `difference`, `is_subset`, and `is_disjoint`
/// are all implemented by walking the tree(s). Time is O(entries) per walk.
#[derive(Clone, Debug)]
pub struct CoordSetN<const N: usize>(CoordSpaceN<N, ()>);

impl<const N: usize> CoordSetN<N> {
    /// Creates an empty set.
    #[inline]
    pub fn new() -> Self {
        Self(CoordSpaceN::new())
    }

    /// Returns the number of coordinates in the set.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the set contains no coordinates.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns `true` if `path` is in the set.
    #[inline]
    pub fn contains(&self, path: CoordPath<N>) -> bool {
        self.0.at_path(&path).is_some()
    }

    /// Inserts `path` into the set.
    /// Returns `true` if `path` was newly inserted (was not already present).
    #[inline]
    pub fn insert(&mut self, path: CoordPath<N>) -> bool {
        self.0.place_path(&path, ()).is_none()
    }

    /// Removes all coordinates from the set.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Removes `path` from the set. Returns `true` if `path` was present.
    #[inline]
    pub fn remove(&mut self, path: CoordPath<N>) -> bool {
        self.0.vacate_path(&path).is_some()
    }

    /// An iterator over all `CoordPath` values in the set, in depth-first
    /// coordinate-ascending order.
    #[inline]
    pub fn iter(&self) -> TreeIter<'_, N, ()> {
        self.0.iter_tree()
    }

    // ── Set operations ──────────────────────────────────────────────

    /// Returns a new set containing all coordinates from `self` and `other`.
    pub fn union(&self, other: &Self) -> Self {
        let mut result = Self::new();
        for (path, _) in self.0.iter_tree() {
            result.0.place_path(&path, ());
        }
        for (path, _) in other.0.iter_tree() {
            result.0.place_path(&path, ());
        }
        result
    }

    /// Returns a new set containing only coordinates present in both sets.
    /// Iterates the smaller set for efficiency.
    pub fn intersection(&self, other: &Self) -> Self {
        let (iter, check) = if self.len() <= other.len() {
            (self, other)
        } else {
            (other, self)
        };
        let mut result = Self::new();
        for (path, _) in iter.0.iter_tree() {
            if check.contains(path) {
                result.0.place_path(&path, ());
            }
        }
        result
    }

    /// Returns a new set containing coordinates in `self` but not in `other`.
    pub fn difference(&self, other: &Self) -> Self {
        let mut result = Self::new();
        for (path, _) in self.0.iter_tree() {
            if !other.contains(path) {
                result.0.place_path(&path, ());
            }
        }
        result
    }

    /// Returns `true` if all coordinates in `self` are also in `other`.
    pub fn is_subset(&self, other: &Self) -> bool {
        for (path, _) in self.0.iter_tree() {
            if !other.contains(path) {
                return false;
            }
        }
        true
    }

    /// Returns `true` if `self` is a superset of `other`.
    #[inline]
    pub fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }

    /// Returns `true` if the two sets have no coordinates in common.
    pub fn is_disjoint(&self, other: &Self) -> bool {
        // Iterate the smaller set for efficiency.
        let (a, b) = if self.len() <= other.len() {
            (self, other)
        } else {
            (other, self)
        };
        for (path, _) in a.0.iter_tree() {
            if b.contains(path) {
                return false;
            }
        }
        true
    }

    /// Returns a new set containing coordinates in `self` or `other` but not both.
    pub fn symmetric_difference(&self, other: &Self) -> Self {
        let mut result = Self::new();
        for (path, _) in self.0.iter_tree() {
            if !other.contains(path) {
                result.0.place_path(&path, ());
            }
        }
        for (path, _) in other.0.iter_tree() {
            if !self.contains(path) {
                result.0.place_path(&path, ());
            }
        }
        result
    }
}

impl<const N: usize> FromIterator<CoordPath<N>> for CoordSetN<N> {
    fn from_iter<I: IntoIterator<Item = CoordPath<N>>>(iter: I) -> Self {
        let mut set = Self::new();
        for path in iter {
            set.insert(path);
        }
        set
    }
}

impl<const N: usize> Default for CoordSetN<N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> PartialEq for CoordSetN<N> {
    fn eq(&self, other: &Self) -> bool {
        // Sets are equal if each is a subset of the other.
        if self.len() != other.len() {
            return false;
        }
        self.is_subset(other)
    }
}
impl<const N: usize> Eq for CoordSetN<N> {}
